use pray_bench::{
    cpu_time_nanos, metadata_with_versions, peak_rss_bytes, scaling_ratio, time_operation,
    BenchProject, ScaleConfig,
};
use pray_core::registry::select_package_version_for_test;
use pray_core::resolve::resolve_project_with_options;
use pray_core::resolve_context::ResolveOptions;
use std::hint::black_box;

const BASELINE_PACKAGES: usize = 5;
const STRESS_PACKAGES: usize = 40;
const MAX_SUPERLINEAR_RATIO: f64 = 2.5;

fn update_options() -> ResolveOptions {
    ResolveOptions {
        refresh_source_revisions: true,
        ignore_locked_versions: true,
        ..ResolveOptions::default()
    }
}

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
        "{label}: scaling ratio {ratio:.2} exceeded {MAX_SUPERLINEAR_RATIO}"
    );
}

#[test]
#[ignore = "scaling guard; run with: cargo test -p pray-bench update_scaling -- --ignored --nocapture"]
fn update_resolve_scales_near_linearly_with_package_count() {
    let baseline = BenchProject::build(ScaleConfig {
        package_count: BASELINE_PACKAGES,
        ..ScaleConfig::default()
    });
    let stress = BenchProject::build(ScaleConfig {
        package_count: STRESS_PACKAGES,
        ..ScaleConfig::default()
    });
    let options = update_options();
    let _ = resolve_project_with_options(&baseline.manifest_path, &options).expect("warmup");
    let baseline_nanos = time_operation(|| {
        let _ = resolve_project_with_options(&baseline.manifest_path, &options).expect("update");
    });
    let stress_nanos = time_operation(|| {
        let _ = resolve_project_with_options(&stress.manifest_path, &options).expect("update");
    });
    let latest_shaped = time_operation(|| {
        let _ = resolve_project_with_options(&stress.manifest_path, &options).expect("preview");
        let _ = resolve_project_with_options(&stress.manifest_path, &options).expect("apply");
    });
    println!(
        "update_resolve packages={}->{} ns={}->{} latest_two_resolves_ns={} ratio={:.2} latest_over_update={:.2} cpu_ns={:?} peak_rss={:?}",
        BASELINE_PACKAGES,
        STRESS_PACKAGES,
        baseline_nanos,
        stress_nanos,
        latest_shaped,
        scaling_ratio(BASELINE_PACKAGES, baseline_nanos, STRESS_PACKAGES, stress_nanos),
        latest_shaped as f64 / stress_nanos as f64,
        cpu_time_nanos(),
        peak_rss_bytes()
    );
    assert_near_linear(
        "update_resolve/packages",
        BASELINE_PACKAGES,
        baseline_nanos,
        STRESS_PACKAGES,
        stress_nanos,
    );
    assert!(
        (1.4..=2.8).contains(&(latest_shaped as f64 / stress_nanos as f64)),
        "update --latest should be about two resolves; got {} vs {}",
        latest_shaped,
        stress_nanos
    );
}

#[test]
#[ignore = "scaling guard; run with: cargo test -p pray-bench update_scaling -- --ignored --nocapture"]
fn version_select_scales_near_linearly_with_published_history() {
    const BASELINE_VERSIONS: usize = 50;
    const STRESS_VERSIONS: usize = 400;
    let baseline = metadata_with_versions("sample/base", BASELINE_VERSIONS);
    let stress = metadata_with_versions("sample/base", STRESS_VERSIONS);
    let _ = select_package_version_for_test(&baseline, ">= 1.0.0", None).expect("warmup");
    let baseline_nanos = time_operation(|| {
        let selected =
            select_package_version_for_test(&baseline, ">= 1.0.0", None).expect("select");
        assert_eq!(black_box(selected.version), "1.49.0");
    });
    let stress_nanos = time_operation(|| {
        let selected = select_package_version_for_test(&stress, ">= 1.0.0", None).expect("select");
        assert_eq!(black_box(selected.version), "1.399.0");
    });
    println!(
        "version_select versions={}->{} ns={}->{} ratio={:.2} metadata_json_bytes={} peak_rss={:?}",
        BASELINE_VERSIONS,
        STRESS_VERSIONS,
        baseline_nanos,
        stress_nanos,
        scaling_ratio(
            BASELINE_VERSIONS,
            baseline_nanos,
            STRESS_VERSIONS,
            stress_nanos
        ),
        serde_json::to_vec(&stress).expect("json").len(),
        peak_rss_bytes()
    );
    assert_near_linear(
        "select_package_version/versions",
        BASELINE_VERSIONS,
        baseline_nanos,
        STRESS_VERSIONS,
        stress_nanos,
    );
}
