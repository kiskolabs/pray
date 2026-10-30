use pray_bench::{peak_rss_bytes, scaling_ratio, time_operation};
use pray_core::package_spec::parse_package_spec;
use std::hint::black_box;

const BASELINE_RELATED: usize = 1_000;
const STRESS_RELATED: usize = 8_000;

#[test]
#[ignore = "scaling guard; run with: cargo test -p pray-bench --test related_scaling -- --ignored --nocapture"]
fn related_prayer_parse_scales_near_linearly() {
    let baseline = prayspec(BASELINE_RELATED);
    let stress = prayspec(STRESS_RELATED);
    parse_package_spec(&baseline).expect("warmup parse");
    let baseline_nanos = time_operation(|| {
        black_box(parse_package_spec(&baseline).expect("baseline parse"));
    });
    let stress_nanos = time_operation(|| {
        black_box(parse_package_spec(&stress).expect("stress parse"));
    });
    let ratio = scaling_ratio(
        BASELINE_RELATED,
        baseline_nanos,
        STRESS_RELATED,
        stress_nanos,
    );
    println!(
        "related count={}->{} bytes={}->{} parse_ns={}->{} ratio={:.2} peak_rss={:?}",
        BASELINE_RELATED,
        STRESS_RELATED,
        baseline.len(),
        stress.len(),
        baseline_nanos,
        stress_nanos,
        ratio,
        peak_rss_bytes()
    );
    assert!(ratio <= 3.0, "related prayer parse ratio {ratio:.2}");
}

fn prayspec(count: usize) -> String {
    let references = (0..count)
        .rev()
        .map(|index| format!("\"bench{index:05}/prayer{index:05}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "Package::Specification.new do |spec|\n  spec.name = \"bench/root\"\n  spec.related = [{references}]\n  spec.files = [\"README.md\"]\nend\n"
    )
}
