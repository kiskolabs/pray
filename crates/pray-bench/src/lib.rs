use pray_core::lockfile::{build_lockfile, write_lockfile, Lockfile, ManagedSpanRecord};
use pray_core::render::{render_project, write_rendered_targets};
use pray_core::resolve::resolve_project;
use pray_core::verify::{find_orphan_marker_findings, verify_project};
use pray_core::PrayResult;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[derive(Debug, Clone, Copy)]
pub struct ScaleConfig {
    pub package_count: usize,
    pub exports_per_package: usize,
    pub lines_per_export: usize,
}

impl Default for ScaleConfig {
    fn default() -> Self {
        Self {
            package_count: 1,
            exports_per_package: 1,
            lines_per_export: 10,
        }
    }
}

pub struct BenchProject {
    pub temp: TempDir,
    pub root: PathBuf,
    pub manifest_path: PathBuf,
}

impl BenchProject {
    pub fn build(config: ScaleConfig) -> Self {
        let temp = tempfile::tempdir().expect("temp directory");
        let root = temp.path().to_path_buf();
        let manifest_path = root.join("Prayfile");
        write_scaled_project(&root, &manifest_path, config);
        Self {
            temp,
            root,
            manifest_path,
        }
    }

    pub fn manifest_text(&self) -> String {
        fs::read_to_string(&self.manifest_path).expect("read Prayfile")
    }

    pub fn prepare_installed_state(&self) -> PrayResult<Lockfile> {
        let project = resolve_project(&self.manifest_path)?;
        let rendered = render_project(&project)?;
        write_rendered_targets(&project, &rendered)?;
        let lockfile = build_lockfile(
            project.lockfile_hash()?,
            &project.manifest.sources,
            &project.manifest.targets,
            &rendered,
            &project.packages,
            &project.source_revisions,
            &project.source_host_keys,
        );
        write_lockfile(&self.root.join("Prayfile.lock"), &lockfile)?;
        Ok(lockfile)
    }

    pub fn verify_clean(&self, lockfile: &Lockfile) -> PrayResult<()> {
        let project = resolve_project(&self.manifest_path)?;
        let report = verify_project(&project, lockfile, false)?;
        assert!(report.is_clean(), "expected clean verification report");
        Ok(())
    }

    pub fn orphan_marker_scan_input(&self, lockfile: &Lockfile) -> OrphanMarkerScanInput {
        let target_path = lockfile
            .managed_span
            .first()
            .map(|span| span.target.clone())
            .unwrap_or_else(|| "INSTRUCTIONS.md".to_string());
        let text = fs::read_to_string(self.root.join(&target_path)).expect("rendered target");
        OrphanMarkerScanInput {
            lines: text.lines().map(str::to_string).collect(),
            spans: lockfile.managed_span.clone(),
            target_path,
        }
    }
}

pub struct OrphanMarkerScanInput {
    lines: Vec<String>,
    spans: Vec<ManagedSpanRecord>,
    target_path: String,
}

impl OrphanMarkerScanInput {
    pub fn managed_span_count(&self) -> usize {
        self.spans.len()
    }

    pub fn run_orphan_marker_scan(&self) -> Vec<pray_core::verify::VerificationFinding> {
        let line_refs: Vec<&str> = self.lines.iter().map(String::as_str).collect();
        let span_refs: Vec<&ManagedSpanRecord> = self.spans.iter().collect();
        find_orphan_marker_findings(&span_refs, &line_refs, &self.target_path)
    }
}

fn write_scaled_project(root: &Path, manifest_path: &Path, config: ScaleConfig) {
    fs::create_dir_all(root.join(".agents")).expect("agents directory");
    fs::write(root.join(".agents/project.md"), "Local guidance\n").expect("local file");

    let mut prayfile = String::from(
        r#"prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
"#,
    );

    for package_index in 0..config.package_count {
        let package_name = format!("bench/pkg-{package_index:04}");
        let package_path = format!("packages/pkg-{package_index:04}");
        write_package(root, &package_path, &package_name, config);
        let export_list = (0..config.exports_per_package)
            .map(|export_index| format!("\"export-{export_index}\""))
            .collect::<Vec<_>>()
            .join(", ");
        prayfile.push_str(&format!(
            "agent \"{package_name}\", path: \"{package_path}\", exports: [{export_list}]\n"
        ));
    }

    prayfile.push_str("local \".agents/project.md\"\n");
    prayfile.push_str("render mode: :managed, conflict: :fail, churn: :minimal\n");
    fs::write(manifest_path, prayfile).expect("write Prayfile");
}

fn write_package(root: &Path, package_path: &str, package_name: &str, config: ScaleConfig) {
    let package_root = root.join(package_path);
    fs::create_dir_all(package_root.join("exports")).expect("package directories");
    fs::write(package_root.join("README.md"), "package readme\n").expect("package readme");

    let mut files = vec!["README.md".to_string()];
    let mut export_entries = Vec::new();
    for export_index in 0..config.exports_per_package {
        let export_name = format!("export-{export_index}");
        let relative_path = format!("exports/{export_name}.md");
        let body = export_body(config.lines_per_export, &export_name);
        fs::write(package_root.join(&relative_path), body).expect("export file");
        files.push(relative_path.clone());
        export_entries.push(format!(
            r#"    "{export_name}" => {{
      type: "fragment",
      path: "{relative_path}",
      summary: "Benchmark export"
    }}"#
        ));
    }

    let prayspec = format!(
        r#"Package::Specification.new do |spec|
  spec.name = "{package_name}"
  spec.version = "1.0.0"
  spec.summary = "benchmark package"
  spec.files = [{files}]
  spec.exports = {{
{exports}
  }}
end
"#,
        files = files
            .iter()
            .map(|file| format!("\"{file}\""))
            .collect::<Vec<_>>()
            .join(", "),
        exports = export_entries.join(",\n")
    );

    let spec_file_name = format!("{}.prayspec", package_name.replace('/', "-"));
    fs::write(package_root.join(spec_file_name), prayspec).expect("write prayspec");
}

fn export_body(line_count: usize, label: &str) -> String {
    let mut body = format!("# {label}\n\n");
    for line_index in 0..line_count {
        body.push_str(&format!("Guidance line {line_index} for {label}.\n"));
    }
    body
}

/// Returns how much wall time grew relative to input growth.
/// A value near 1.0 suggests linear scaling; much larger values hint at superlinear cost.
pub fn scaling_ratio(baseline_size: usize, baseline_nanos: u128, size: usize, nanos: u128) -> f64 {
    if baseline_size == 0 || size == 0 || baseline_nanos == 0 {
        return f64::NAN;
    }
    let size_factor = size as f64 / baseline_size as f64;
    let time_factor = nanos as f64 / baseline_nanos as f64;
    time_factor / size_factor
}

pub fn time_operation<F>(mut operation: F) -> u128
where
    F: FnMut(),
{
    let start = std::time::Instant::now();
    operation();
    start.elapsed().as_nanos()
}
