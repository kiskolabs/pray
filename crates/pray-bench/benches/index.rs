use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use pray_bench::{compact_index, compact_index_json, metadata_with_versions, write_search_fixture};
use pray_core::registry::select_package_version_for_test;
use pray_core::registry_search::{search_local_registry, search_registry_index_names};
use tempfile::tempdir;

fn package_counts() -> Vec<usize> {
    vec![100, 1_000, 10_000, 50_000]
}

fn version_counts() -> Vec<usize> {
    vec![10, 50, 200, 1_000]
}

fn bench_index_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_index/parse");
    for package_count in package_counts() {
        let json = compact_index_json(package_count);
        group.throughput(Throughput::Bytes(json.len() as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(package_count),
            &json,
            |bencher, json| {
                bencher.iter(|| {
                    let index: pray_core::registry::RegistryIndex =
                        serde_json::from_str(json).expect("parse index");
                    black_box(index.packages.len())
                });
            },
        );
    }
    group.finish();
}

fn bench_index_name_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_index/name_search");
    for package_count in package_counts() {
        let index = compact_index(package_count);
        group.throughput(Throughput::Elements(package_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(package_count),
            &index,
            |bencher, index| {
                bencher.iter(|| black_box(search_registry_index_names(index, "pkg0001")));
            },
        );
    }
    group.finish();
}

fn bench_summary_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_index/summary_search");
    group.sample_size(30);
    for package_count in [20usize, 80, 160] {
        let temp = tempdir().expect("search fixture");
        write_search_fixture(temp.path(), package_count, true);
        group.throughput(Throughput::Elements(package_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(package_count),
            temp.path(),
            |bencher, root| {
                bencher.iter(|| {
                    let hits = search_local_registry(root, "pkg", true).expect("search");
                    black_box(hits.len())
                });
            },
        );
    }
    group.finish();
}

fn bench_version_select(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry_metadata/select_version");
    for version_count in version_counts() {
        let metadata = metadata_with_versions("sample/base", version_count);
        group.throughput(Throughput::Elements(version_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(version_count),
            &metadata,
            |bencher, metadata| {
                bencher.iter(|| {
                    black_box(
                        select_package_version_for_test(metadata, ">= 1.0.0", None)
                            .expect("select"),
                    )
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_index_parse,
    bench_index_name_search,
    bench_summary_search,
    bench_version_select
);
criterion_main!(benches);
