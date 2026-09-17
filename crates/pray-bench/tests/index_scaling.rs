use pray_bench::{
    compact_index, compact_index_json, cpu_time_nanos, names_heap_bytes, peak_rss_bytes,
    scaling_ratio, time_operation, write_search_fixture,
};
use pray_core::registry::RegistryIndex;
use pray_core::registry_search::{search_local_registry, search_registry_index_names};
use pray_core::resource_limits::MAX_HTTP_RESPONSE_BYTES;
use std::hint::black_box;
use tempfile::tempdir;

const BASELINE_PACKAGES: usize = 5_000;
const STRESS_PACKAGES: usize = 40_000;
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
        "{label}: scaling ratio {ratio:.2} exceeded {MAX_SUPERLINEAR_RATIO}"
    );
}

#[test]
fn compact_index_stays_well_under_http_get_ceiling_at_catalog_sizes() {
    let json = compact_index_json(10_000);
    let bytes_per_name = json.len() as f64 / 10_000.0;
    let http_cliff = (MAX_HTTP_RESPONSE_BYTES as f64 / bytes_per_name) as u64;
    println!(
        "index_profile packages=10000 json_bytes={} bytes_per_name={:.1} http_get_cliff_packages={} peak_rss={:?}",
        json.len(),
        bytes_per_name,
        http_cliff,
        peak_rss_bytes()
    );
    assert!(json.len() < 512 * 1024);
    assert!(http_cliff > 1_500_000);
}

#[test]
#[ignore = "scaling guard; run with: cargo test -p pray-bench index_scaling -- --ignored --nocapture"]
fn index_parse_and_name_search_scale_near_linearly() {
    let baseline_json = compact_index_json(BASELINE_PACKAGES);
    let stress_json = compact_index_json(STRESS_PACKAGES);
    let _ = serde_json::from_str::<RegistryIndex>(&baseline_json).expect("warmup parse");
    let baseline_parse = time_operation(|| {
        let _ = black_box(serde_json::from_str::<RegistryIndex>(&baseline_json).expect("parse"));
    });
    let stress_parse = time_operation(|| {
        let _ = black_box(serde_json::from_str::<RegistryIndex>(&stress_json).expect("parse"));
    });
    let baseline_index = compact_index(BASELINE_PACKAGES);
    let stress_index = compact_index(STRESS_PACKAGES);
    let baseline_search = time_operation(|| {
        let _ = black_box(search_registry_index_names(&baseline_index, "pkg0001"));
    });
    let stress_search = time_operation(|| {
        let _ = black_box(search_registry_index_names(&stress_index, "pkg0001"));
    });
    println!(
        "index_parse json_bytes={}->{} heap_bytes={}->{} parse_ns={}->{} search_ns={}->{} cpu_ns={:?} peak_rss={:?}",
        baseline_json.len(),
        stress_json.len(),
        names_heap_bytes(&baseline_index),
        names_heap_bytes(&stress_index),
        baseline_parse,
        stress_parse,
        baseline_search,
        stress_search,
        cpu_time_nanos(),
        peak_rss_bytes()
    );
    assert_near_linear(
        "index_parse/packages",
        BASELINE_PACKAGES,
        baseline_parse,
        STRESS_PACKAGES,
        stress_parse,
    );
    assert_near_linear(
        "index_search/packages",
        BASELINE_PACKAGES,
        baseline_search,
        STRESS_PACKAGES,
        stress_search,
    );
}

#[test]
#[ignore = "scaling guard; run with: cargo test -p pray-bench index_scaling -- --ignored --nocapture"]
fn summary_search_cost_tracks_match_count_not_index_parse() {
    const BASELINE_HITS: usize = 20;
    const STRESS_HITS: usize = 160;
    let baseline_root = tempdir().expect("baseline");
    let stress_root = tempdir().expect("stress");
    write_search_fixture(baseline_root.path(), BASELINE_HITS, true);
    write_search_fixture(stress_root.path(), STRESS_HITS, true);
    let _ = search_local_registry(baseline_root.path(), "pkg", true).expect("warmup");
    let baseline_nanos = time_operation(|| {
        let hits = search_local_registry(baseline_root.path(), "pkg", true).expect("search");
        assert_eq!(hits.len(), BASELINE_HITS);
    });
    let stress_nanos = time_operation(|| {
        let hits = search_local_registry(stress_root.path(), "pkg", true).expect("search");
        assert_eq!(hits.len(), STRESS_HITS);
    });
    let names_only = time_operation(|| {
        let hits = search_local_registry(stress_root.path(), "pkg", false).expect("names");
        assert_eq!(hits.len(), STRESS_HITS);
    });
    println!(
        "summary_search hits={}->{} with_summary_ns={}->{} names_only_ns={} ratio={:.2} peak_rss={:?}",
        BASELINE_HITS,
        STRESS_HITS,
        baseline_nanos,
        stress_nanos,
        names_only,
        scaling_ratio(BASELINE_HITS, baseline_nanos, STRESS_HITS, stress_nanos),
        peak_rss_bytes()
    );
    assert_near_linear(
        "search_summaries/hits",
        BASELINE_HITS,
        baseline_nanos,
        STRESS_HITS,
        stress_nanos,
    );
    assert!(
        stress_nanos > names_only.saturating_mul(4),
        "summary search should dominate names-only scan: summary={stress_nanos} names={names_only}"
    );
}
