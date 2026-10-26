use pray_bench::{cpu_time_nanos, peak_rss_bytes};
use pray_core::embed::{
    build_lockfile, parse_manifest, render_project, resolve_project, serialize_lockfile,
};
use pray_core::hashing::sha256_prefixed;
use std::{env, fs, hint::black_box, path::Path, time::Instant};

const DEFAULT_ITERATIONS: usize = 500;
const DEFAULT_WARMUP: usize = 20;

struct Measurement {
    p50: f64,
    p95: f64,
}

fn measure(iterations: usize, warmup: usize, mut operation: impl FnMut()) -> Measurement {
    for _ in 0..warmup {
        operation();
    }
    let mut samples: Vec<f64> = (0..iterations)
        .map(|_| {
            let started = Instant::now();
            operation();
            started.elapsed().as_nanos() as f64 / 1_000.0
        })
        .collect();
    samples.sort_by(f64::total_cmp);
    let middle = iterations / 2;
    let p50 = if iterations.is_multiple_of(2) {
        (samples[middle - 1] + samples[middle]) / 2.0
    } else {
        samples[middle]
    };
    Measurement {
        p50,
        p95: samples[(iterations as f64 * 0.95).ceil() as usize - 1],
    }
}

fn print_measurement(name: &str, measurement: &Measurement) {
    println!(
        "{name} p50_us={:.2} p95_us={:.2} ops_s={:.0}",
        measurement.p50,
        measurement.p95,
        1_000_000.0 / measurement.p50
    );
}

fn package_identity(index: usize) -> (String, String) {
    if index == 0 {
        return ("sample/base".to_string(), "packages/base".to_string());
    }
    (
        format!("sample/pkg-{index:04}"),
        format!("packages/pkg-{index:04}"),
    )
}

fn build_prayspec(name: &str) -> String {
    format!(
        "Package::Specification.new do |spec|\n  spec.name = \"{name}\"\n  spec.version = \"1.0.0\"\n  spec.files = [\"exports/a.md\"]\n  spec.exports = {{ \"a\" => {{ type: \"fragment\", path: \"exports/a.md\" }} }}\nend\n"
    )
}

fn write_fixture(root: &Path, package_count: usize) -> std::io::Result<(String, usize)> {
    let mut manifest = String::from("prayfile \"1\"\ncompose \"A.md\" do\n");
    let mut input_bytes = 0;
    for index in 0..package_count {
        let (name, path) = package_identity(index);
        manifest.push_str(&format!("  pray \"{name}\", path: \"{path}\"\n"));
        let prayspec = build_prayspec(&name);
        fs::create_dir_all(root.join(&path).join("exports"))?;
        fs::write(root.join(&path).join("package.prayspec"), &prayspec)?;
        fs::write(root.join(&path).join("exports/a.md"), "A\n")?;
        input_bytes += prayspec.len() + 2;
    }
    manifest.push_str("end\n");
    input_bytes += manifest.len();
    fs::write(root.join("Prayfile"), &manifest)?;
    Ok((manifest, input_bytes))
}

fn env_usize(name: &str, default: usize, allow_zero: bool) -> Result<usize, String> {
    let value = env::var(name).map_or(Ok(default), |text| {
        text.parse::<usize>()
            .map_err(|error| format!("{name}: {error}"))
    })?;
    if !allow_zero && value == 0 {
        return Err(format!("{name} must be positive"));
    }
    Ok(value)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let iterations = env_usize("PRAY_BENCH_ITERATIONS", DEFAULT_ITERATIONS, false)?;
    let warmup = env_usize("PRAY_BENCH_WARMUP", DEFAULT_WARMUP, true)?;
    let package_count = env_usize("PRAY_BENCH_PACKAGES", 1, false)?;
    let temp = tempfile::tempdir()?;
    let root = temp.path();
    let (manifest, input_bytes) = write_fixture(root, package_count)?;
    let manifest_path = root.join("Prayfile");
    let project = resolve_project(&manifest_path)?;
    let rendered = render_project(&project)?;
    let benchmark_started = Instant::now();
    let cpu_started = cpu_time_nanos();
    let parse = measure(iterations, warmup, || {
        black_box(parse_manifest(black_box(&manifest)).expect("parse"));
    });
    let resolve = measure(iterations, warmup, || {
        black_box(resolve_project(black_box(&manifest_path)).expect("resolve"));
    });
    let render = measure(iterations, warmup, || {
        black_box(render_project(black_box(&project)).expect("render"));
    });
    let lock = measure(iterations, warmup, || {
        let lockfile = build_lockfile(
            project.manifest_hash.clone(),
            project.environment.clone(),
            &project.project_root,
            &project.manifest.sources,
            &project.manifest.targets,
            &rendered,
            &project.packages,
            &project.source_revisions,
            &project.source_host_keys,
        );
        black_box(serialize_lockfile(&lockfile).expect("lock"));
    });
    print_measurement("parse", &parse);
    print_measurement("resolve", &resolve);
    print_measurement("render", &render);
    print_measurement("lock", &lock);
    let rendered_bytes = rendered
        .iter()
        .map(|target| target.content.as_str())
        .collect::<String>();
    println!(
        "fixture packages={package_count} input_bytes={input_bytes} output_bytes={} output_sha256={}",
        rendered_bytes.len(),
        sha256_prefixed(rendered_bytes.as_bytes())
    );
    let cpu_nanos = cpu_time_nanos()
        .zip(cpu_started)
        .map(|(finished, started)| finished.saturating_sub(started))
        .unwrap_or_default();
    println!(
        "resources wall_ms={:.2} cpu_ms={:.2} peak_rss_bytes={} iterations={iterations} warmup={warmup}",
        benchmark_started.elapsed().as_secs_f64() * 1_000.0,
        cpu_nanos as f64 / 1_000_000.0,
        peak_rss_bytes().unwrap_or_default()
    );
    Ok(())
}
