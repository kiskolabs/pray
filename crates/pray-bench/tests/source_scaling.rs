use pray_bench::{cpu_time_nanos, peak_rss_bytes, scaling_ratio, time_operation};
use pray_core::resolve::resolve_project_with_options;
use pray_core::resolve_context::ResolveOptions;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const BASELINE_SOURCES: usize = 2;
const STRESS_SOURCES: usize = 8;
const MAX_SUPERLINEAR_RATIO: f64 = 2.5;

#[test]
#[ignore = "scaling guard; run with: cargo test -p pray-bench source_scaling -- --ignored --nocapture"]
fn update_resolve_scales_near_linearly_with_git_source_count() {
    let options = ResolveOptions {
        refresh_source_revisions: true,
        ignore_locked_versions: true,
        ..ResolveOptions::default()
    };
    let baseline = GitSourceProject::build(BASELINE_SOURCES);
    let baseline_cold = time_operation(|| {
        let _ = resolve_project_with_options(&baseline.manifest_path, &options).expect("cold");
    });
    let baseline_warm = time_operation(|| {
        let _ = resolve_project_with_options(&baseline.manifest_path, &options).expect("warm");
    });
    let stress = GitSourceProject::build(STRESS_SOURCES);
    let stress_cold = time_operation(|| {
        let _ = resolve_project_with_options(&stress.manifest_path, &options).expect("cold");
    });
    let stress_warm = time_operation(|| {
        let _ = resolve_project_with_options(&stress.manifest_path, &options).expect("warm");
    });
    println!(
        "git_sources={}->{} cold_ns={}->{} warm_ns={}->{} cold_ratio={:.2} warm_ratio={:.2} cpu_ns={:?} peak_rss={:?}",
        BASELINE_SOURCES,
        STRESS_SOURCES,
        baseline_cold,
        stress_cold,
        baseline_warm,
        stress_warm,
        scaling_ratio(BASELINE_SOURCES, baseline_cold, STRESS_SOURCES, stress_cold),
        scaling_ratio(BASELINE_SOURCES, baseline_warm, STRESS_SOURCES, stress_warm),
        cpu_time_nanos(),
        peak_rss_bytes()
    );
    let ratio = scaling_ratio(BASELINE_SOURCES, baseline_warm, STRESS_SOURCES, stress_warm);
    assert!(
        ratio <= MAX_SUPERLINEAR_RATIO,
        "git source scaling ratio {ratio:.2} exceeded {MAX_SUPERLINEAR_RATIO}"
    );
}

struct GitSourceProject {
    root: PathBuf,
    manifest_path: PathBuf,
}

impl GitSourceProject {
    fn build(source_count: usize) -> Self {
        let root = unique_temp("pray-git-source-scale");
        std::env::set_var("PRAY_CACHE", root.join("cache"));
        write_path_package(&root);
        let mut prayfile = String::from("prayfile \"1\"\n");
        for index in 0..source_count {
            let repo = init_git_repo(&root.join(format!("catalog-{index}")), b"catalog\n");
            prayfile.push_str(&format!(
                "source \"catalog-{index}\", \"git+file://{repo}\"\n"
            ));
        }
        prayfile.push_str("pray \"bench/pkg\", path: \"packages/pkg\"\n");
        let manifest_path = root.join("Prayfile");
        fs::write(&manifest_path, prayfile).expect("Prayfile");
        Self {
            root,
            manifest_path,
        }
    }
}

impl Drop for GitSourceProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn write_path_package(root: &Path) {
    let package = root.join("packages/pkg");
    fs::create_dir_all(&package).expect("package");
    fs::write(
        package.join("bench-pkg.prayspec"),
        "Package::Specification.new do |spec|\n  spec.name = \"bench/pkg\"\n  spec.version = \"1.0.0\"\n  spec.files = [\"bench-pkg.prayspec\"]\nend\n",
    )
    .expect("prayspec");
}

fn init_git_repo(path: &Path, blob: &[u8]) -> String {
    fs::create_dir_all(path).expect("repo");
    fs::write(path.join("catalog.bin"), blob).expect("blob");
    git(path, &["init", "-b", "main"]);
    git(path, &["add", "-A"]);
    git(
        path,
        &[
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=/dev/null",
            "commit",
            "-m",
            "catalog",
        ],
    );
    path.canonicalize()
        .expect("canonicalize")
        .to_string_lossy()
        .into_owned()
}

fn git(directory: &Path, arguments: &[&str]) {
    let output = Command::new("git")
        .current_dir(directory)
        .env("GIT_AUTHOR_NAME", "pray")
        .env("GIT_AUTHOR_EMAIL", "pray@example.com")
        .env("GIT_COMMITTER_NAME", "pray")
        .env("GIT_COMMITTER_EMAIL", "pray@example.com")
        .args(arguments)
        .output()
        .expect("git");
    assert!(
        output.status.success(),
        "git {arguments:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn unique_temp(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{prefix}-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos(),
        prefix.len()
    ))
}
