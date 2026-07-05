use pray_bench::{scaling_ratio, time_operation, BenchProject, OrphanMarkerScanInput, ScaleConfig};
use pray_core::render::render_project;
use pray_core::resolve::resolve_project;
use pray_core::verify::verify_project;

const BASELINE_SIZE: usize = 5;
const STRESS_SIZE: usize = 40;
const MAX_SUPERLINEAR_RATIO: f64 = 2.5;

fn assert_near_linear(
    label: &str,
    baseline_size: usize,
    baseline_nanos: u128,
    size: usize,
    nanos: u128,
) {
    let ratio = scaling_ratio(baseline_size, baseline_nanos, size, nanos);
    assert!(
        ratio <= MAX_SUPERLINEAR_RATIO,
        "{label}: scaling ratio {ratio:.2} exceeded {MAX_SUPERLINEAR_RATIO} (baseline {baseline_size} -> {size})"
    );
}

#[test]
#[ignore = "scaling guard; run with: cargo test -p pray-bench scaling -- --ignored --nocapture"]
fn resolve_project_scales_near_linearly_with_package_count() {
    let baseline = BenchProject::build(ScaleConfig {
        package_count: BASELINE_SIZE,
        ..ScaleConfig::default()
    });
    let stress = BenchProject::build(ScaleConfig {
        package_count: STRESS_SIZE,
        ..ScaleConfig::default()
    });

    let baseline_nanos = time_operation(|| {
        let _ = resolve_project(&baseline.manifest_path).expect("resolve");
    });
    let stress_nanos = time_operation(|| {
        let _ = resolve_project(&stress.manifest_path).expect("resolve");
    });

    println!(
        "resolve_project packages: baseline={baseline_nanos}ns stress={stress_nanos}ns ratio={}",
        scaling_ratio(BASELINE_SIZE, baseline_nanos, STRESS_SIZE, stress_nanos)
    );
    assert_near_linear(
        "resolve_project/packages",
        BASELINE_SIZE,
        baseline_nanos,
        STRESS_SIZE,
        stress_nanos,
    );
}

#[test]
#[ignore = "scaling guard; run with: cargo test -p pray-bench scaling -- --ignored --nocapture"]
fn verify_project_scales_near_linearly_with_managed_spans() {
    let baseline = BenchProject::build(ScaleConfig {
        package_count: BASELINE_SIZE,
        ..ScaleConfig::default()
    });
    let stress = BenchProject::build(ScaleConfig {
        package_count: STRESS_SIZE,
        ..ScaleConfig::default()
    });
    let baseline_lockfile = baseline.prepare_installed_state().expect("install");
    let stress_lockfile = stress.prepare_installed_state().expect("install");
    let baseline_resolved = resolve_project(&baseline.manifest_path).expect("resolve");
    let stress_resolved = resolve_project(&stress.manifest_path).expect("resolve");

    let baseline_nanos = time_operation(|| {
        let _ = verify_project(&baseline_resolved, &baseline_lockfile, false).expect("verify");
    });
    let stress_nanos = time_operation(|| {
        let _ = verify_project(&stress_resolved, &stress_lockfile, false).expect("verify");
    });

    println!(
        "verify_project spans: baseline={baseline_nanos}ns stress={stress_nanos}ns ratio={}",
        scaling_ratio(BASELINE_SIZE, baseline_nanos, STRESS_SIZE, stress_nanos)
    );
    assert_near_linear(
        "verify_project/packages",
        BASELINE_SIZE,
        baseline_nanos,
        STRESS_SIZE,
        stress_nanos,
    );
}

#[test]
#[ignore = "scaling guard; run with: cargo test -p pray-bench scaling -- --ignored --nocapture"]
fn render_project_scales_near_linearly_with_export_lines() {
    let baseline = BenchProject::build(ScaleConfig {
        package_count: 1,
        exports_per_package: 1,
        lines_per_export: 100,
    });
    let stress = BenchProject::build(ScaleConfig {
        package_count: 1,
        exports_per_package: 1,
        lines_per_export: 800,
    });
    let baseline_resolved = resolve_project(&baseline.manifest_path).expect("resolve");
    let stress_resolved = resolve_project(&stress.manifest_path).expect("resolve");

    let baseline_nanos = time_operation(|| {
        let _ = render_project(&baseline_resolved).expect("render");
    });
    let stress_nanos = time_operation(|| {
        let _ = render_project(&stress_resolved).expect("render");
    });

    println!(
        "render_project lines: baseline={baseline_nanos}ns stress={stress_nanos}ns ratio={}",
        scaling_ratio(100, baseline_nanos, 800, stress_nanos)
    );
    assert_near_linear(
        "render_project/export_lines",
        100,
        baseline_nanos,
        800,
        stress_nanos,
    );
}

#[test]
#[ignore = "scaling guard; run with: cargo test -p pray-bench -- --ignored --nocapture"]
fn orphan_marker_scan_scales_near_linearly_with_managed_span_count() {
    const BASELINE_SPANS: usize = 10;
    const STRESS_SPANS: usize = 40;

    let baseline = BenchProject::build(ScaleConfig {
        package_count: 1,
        exports_per_package: BASELINE_SPANS,
        lines_per_export: 1,
    });
    let stress = BenchProject::build(ScaleConfig {
        package_count: 1,
        exports_per_package: STRESS_SPANS,
        lines_per_export: 1,
    });
    let baseline_lockfile = baseline.prepare_installed_state().expect("install");
    let stress_lockfile = stress.prepare_installed_state().expect("install");
    let baseline_input = baseline.orphan_marker_scan_input(&baseline_lockfile);
    let stress_input = stress.orphan_marker_scan_input(&stress_lockfile);

    let baseline_nanos = time_orphan_marker_scan(&baseline_input);
    let stress_nanos = time_orphan_marker_scan(&stress_input);
    let ratio = scaling_ratio(BASELINE_SPANS, baseline_nanos, STRESS_SPANS, stress_nanos);

    println!(
        "orphan_marker_scan spans: baseline={baseline_nanos}ns stress={stress_nanos}ns ratio={ratio:.2}"
    );
    assert_near_linear(
        "verify_project/orphan_marker_scan",
        BASELINE_SPANS,
        baseline_nanos,
        STRESS_SPANS,
        stress_nanos,
    );
}

fn time_orphan_marker_scan(input: &OrphanMarkerScanInput) -> u128 {
    time_operation(|| {
        let _ = input.run_orphan_marker_scan();
    })
}
