use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use pray_bench::{BenchProject, ScaleConfig};
use pray_core::lockfile::read_lockfile;
use pray_core::manifest::parse_manifest;
use pray_core::render::render_project;
use pray_core::resolve::resolve_project;
use pray_core::verify::drift_project;
use pray_core::verify::verify_project;

fn package_counts() -> Vec<usize> {
    vec![1, 2, 5, 10, 20, 50]
}

fn export_counts() -> Vec<usize> {
    vec![1, 2, 5, 10, 20, 50]
}

fn export_line_counts() -> Vec<usize> {
    vec![10, 50, 100, 500, 1_000]
}

fn managed_span_counts() -> Vec<usize> {
    vec![10, 25, 50, 100, 200]
}

fn bench_parse_manifest(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_manifest/packages");
    for package_count in package_counts() {
        let project = BenchProject::build(ScaleConfig {
            package_count,
            ..ScaleConfig::default()
        });
        let manifest_text = project.manifest_text();
        group.throughput(Throughput::Elements(package_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(package_count),
            &manifest_text,
            |bencher, text| {
                bencher.iter(|| black_box(parse_manifest(text).expect("parse manifest")));
            },
        );
    }
    group.finish();
}

fn bench_resolve_project(c: &mut Criterion) {
    let mut group = c.benchmark_group("resolve_project/packages");
    for package_count in package_counts() {
        let project = BenchProject::build(ScaleConfig {
            package_count,
            ..ScaleConfig::default()
        });
        group.throughput(Throughput::Elements(package_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(package_count),
            &project.manifest_path,
            |bencher, manifest_path| {
                bencher.iter(|| black_box(resolve_project(manifest_path).expect("resolve")));
            },
        );
    }
    group.finish();
}

fn bench_render_project(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_project/packages");
    for package_count in package_counts() {
        let project = BenchProject::build(ScaleConfig {
            package_count,
            ..ScaleConfig::default()
        });
        let resolved = resolve_project(&project.manifest_path).expect("resolve");
        group.throughput(Throughput::Elements(package_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(package_count),
            &resolved,
            |bencher, resolved_project| {
                bencher.iter(|| black_box(render_project(resolved_project).expect("render")));
            },
        );
    }
    group.finish();
}

fn bench_verify_project(c: &mut Criterion) {
    let mut group = c.benchmark_group("verify_project/packages");
    for package_count in package_counts() {
        let project = BenchProject::build(ScaleConfig {
            package_count,
            ..ScaleConfig::default()
        });
        let lockfile = project.prepare_installed_state().expect("install state");
        let resolved = resolve_project(&project.manifest_path).expect("resolve");
        group.throughput(Throughput::Elements(package_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(package_count),
            &(resolved, lockfile),
            |bencher, input| {
                let (resolved_project, lockfile) = input;
                bencher.iter(|| {
                    black_box(verify_project(resolved_project, lockfile, false).expect("verify"))
                });
            },
        );
    }
    group.finish();
}

fn bench_exports_per_package(c: &mut Criterion) {
    let mut group = c.benchmark_group("resolve_project/exports_per_package");
    for export_count in export_counts() {
        let project = BenchProject::build(ScaleConfig {
            package_count: 1,
            exports_per_package: export_count,
            ..ScaleConfig::default()
        });
        group.throughput(Throughput::Elements(export_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(export_count),
            &project.manifest_path,
            |bencher, manifest_path| {
                bencher.iter(|| black_box(resolve_project(manifest_path).expect("resolve")));
            },
        );
    }
    group.finish();

    let mut render_group = c.benchmark_group("render_project/exports_per_package");
    for export_count in export_counts() {
        let project = BenchProject::build(ScaleConfig {
            package_count: 1,
            exports_per_package: export_count,
            ..ScaleConfig::default()
        });
        let resolved = resolve_project(&project.manifest_path).expect("resolve");
        render_group.throughput(Throughput::Elements(export_count as u64));
        render_group.bench_with_input(
            BenchmarkId::from_parameter(export_count),
            &resolved,
            |bencher, resolved_project| {
                bencher.iter(|| black_box(render_project(resolved_project).expect("render")));
            },
        );
    }
    render_group.finish();
}

fn bench_export_body_lines(c: &mut Criterion) {
    let mut render_group = c.benchmark_group("render_project/export_lines");
    for line_count in export_line_counts() {
        let project = BenchProject::build(ScaleConfig {
            package_count: 1,
            exports_per_package: 1,
            lines_per_export: line_count,
        });
        let resolved = resolve_project(&project.manifest_path).expect("resolve");
        render_group.throughput(Throughput::Elements(line_count as u64));
        render_group.bench_with_input(
            BenchmarkId::from_parameter(line_count),
            &resolved,
            |bencher, resolved_project| {
                bencher.iter(|| black_box(render_project(resolved_project).expect("render")));
            },
        );
    }
    render_group.finish();

    let mut verify_group = c.benchmark_group("verify_project/export_lines");
    for line_count in export_line_counts() {
        let project = BenchProject::build(ScaleConfig {
            package_count: 1,
            exports_per_package: 1,
            lines_per_export: line_count,
        });
        let lockfile = project.prepare_installed_state().expect("install state");
        let resolved = resolve_project(&project.manifest_path).expect("resolve");
        verify_group.throughput(Throughput::Elements(line_count as u64));
        verify_group.bench_with_input(
            BenchmarkId::from_parameter(line_count),
            &(resolved, lockfile),
            |bencher, input| {
                let (resolved_project, lockfile) = input;
                bencher.iter(|| {
                    black_box(verify_project(resolved_project, lockfile, false).expect("verify"))
                });
            },
        );
    }
    verify_group.finish();
}

fn bench_lockfile_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("lockfile_roundtrip/packages");
    for package_count in package_counts() {
        let project = BenchProject::build(ScaleConfig {
            package_count,
            ..ScaleConfig::default()
        });
        let lockfile = project.prepare_installed_state().expect("install state");
        let lockfile_path = project.root.join("Prayfile.lock");
        group.throughput(Throughput::Elements(package_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(package_count),
            &(lockfile, lockfile_path),
            |bencher, input| {
                let (lockfile, path) = input;
                bencher.iter(|| {
                    black_box(lockfile.serialized().expect("serialize lockfile"));
                    black_box(read_lockfile(path).expect("read lockfile"));
                });
            },
        );
    }
    group.finish();
}

fn bench_install_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("install_pipeline/packages");
    for package_count in package_counts() {
        let project = BenchProject::build(ScaleConfig {
            package_count,
            ..ScaleConfig::default()
        });
        group.throughput(Throughput::Elements(package_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(package_count),
            &project.manifest_path,
            |bencher, manifest_path| {
                bencher.iter(|| {
                    let project = resolve_project(manifest_path).expect("resolve");
                    let rendered = render_project(&project).expect("render");
                    black_box(project.lockfile_hash().expect("manifest hash for lockfile"));
                    black_box(rendered);
                });
            },
        );
    }
    group.finish();
}

fn bench_orphan_marker_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("verify_project/orphan_marker_scan");
    group.sample_size(50);
    for managed_span_count in managed_span_counts() {
        let project = BenchProject::build(ScaleConfig {
            package_count: 1,
            exports_per_package: managed_span_count,
            lines_per_export: 1,
        });
        let lockfile = project.prepare_installed_state().expect("install state");
        let scan_input = project.orphan_marker_scan_input(&lockfile);
        assert_eq!(
            scan_input.managed_span_count(),
            managed_span_count,
            "fixture managed span count"
        );
        group.throughput(Throughput::Elements(managed_span_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(managed_span_count),
            &scan_input,
            |bencher, input| {
                bencher.iter(|| black_box(input.run_orphan_marker_scan()));
            },
        );
    }
    group.finish();
}

fn bench_drift_project(c: &mut Criterion) {
    let mut group = c.benchmark_group("drift_project/packages");
    for package_count in package_counts() {
        let project = BenchProject::build(ScaleConfig {
            package_count,
            ..ScaleConfig::default()
        });
        let lockfile = project.prepare_installed_state().expect("install state");
        let resolved = resolve_project(&project.manifest_path).expect("resolve");
        group.throughput(Throughput::Elements(package_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(package_count),
            &(resolved, lockfile),
            |bencher, input| {
                let (resolved_project, lockfile) = input;
                bencher.iter(|| {
                    black_box(drift_project(resolved_project, lockfile).expect("drift"));
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_parse_manifest,
    bench_resolve_project,
    bench_render_project,
    bench_verify_project,
    bench_exports_per_package,
    bench_export_body_lines,
    bench_lockfile_roundtrip,
    bench_install_pipeline,
    bench_orphan_marker_scan,
    bench_drift_project
);
criterion_main!(benches);
