use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn run_pray(repo: &Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_pray"))
        .args(arguments)
        .current_dir(repo)
        .output()
        .expect("run pray")
}

pub fn temporary_directory(prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let suffix = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = std::env::temp_dir().join(format!("{prefix}-{stamp}-{suffix}"));
    fs::create_dir_all(&path).expect("temp dir");
    path
}

pub fn create_fixture(repo: &Path) {
    fs::create_dir_all(repo.join("packages/base/exports")).expect("fixture directories");
    fs::create_dir_all(repo.join("agent/local")).expect("local directories");

    fs::write(
        repo.join("Prayfile"),
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
agent "sample/base", "~> 1.4", path: "packages/base"
local "agent/local/project.md"
render mode: :managed, conflict: :fail, churn: :minimal
"#,
    )
    .expect("write Prayfile");

    fs::write(
        repo.join("packages/base/sample-base.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.4.3"
  spec.summary = "shared guidance"
  spec.files = ["README.md", "exports/testing-basics.md", "exports/security-basics.md"]
  spec.exports = {
    "testing-basics" => {
      type: "fragment",
      path: "exports/testing-basics.md",
      summary: "Testing guidance"
    },
    "security-basics" => {
      type: "fragment",
      path: "exports/security-basics.md",
      summary: "Security guidance"
    }
  }
end
"#,
    )
    .expect("write prayspec");

    fs::write(repo.join("packages/base/README.md"), "package readme\n").expect("write readme");
    fs::write(
        repo.join("packages/base/exports/testing-basics.md"),
        "Testing guidance\n",
    )
    .expect("write export");
    fs::write(
        repo.join("packages/base/exports/security-basics.md"),
        "Security guidance\n",
    )
    .expect("write export");
    fs::write(repo.join("agent/local/project.md"), "Local guidance\n").expect("write local");
}

pub fn create_add_fixture(repo: &Path) {
    fs::create_dir_all(repo.join("packages/base/exports")).expect("fixture directories");
    fs::write(
        repo.join("Prayfile"),
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
render mode: :managed, conflict: :fail, churn: :minimal
"#,
    )
    .expect("write Prayfile");

    fs::write(
        repo.join("packages/base/sample-base.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.4.3"
  spec.summary = "shared guidance"
  spec.files = ["README.md", "exports/testing-basics.md"]
  spec.exports = {
    "testing-basics" => {
      type: "fragment",
      path: "exports/testing-basics.md",
      summary: "Testing guidance"
    }
  }
end
"#,
    )
    .expect("write prayspec");

    fs::write(repo.join("packages/base/README.md"), "package readme\n").expect("write readme");
    fs::write(
        repo.join("packages/base/exports/testing-basics.md"),
        "Testing guidance\n",
    )
    .expect("write export");
}

pub fn create_tree_fixture(repo: &Path) {
    fs::create_dir_all(repo.join("packages/base/exports")).expect("base directories");
    fs::create_dir_all(repo.join("packages/common/exports")).expect("common directories");
    fs::write(
        repo.join("Prayfile"),
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
agent "sample/base", path: "packages/base"
agent "sample/common", path: "packages/common"
render mode: :managed, conflict: :fail, churn: :minimal
"#,
    )
    .expect("write Prayfile");

    fs::write(
        repo.join("packages/base/sample-base.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.4.3"
  spec.summary = "shared guidance"
  spec.files = ["README.md", "exports/testing-basics.md"]
  spec.exports = {
    "testing-basics" => {
      type: "fragment",
      path: "exports/testing-basics.md",
      summary: "Testing guidance"
    }
  }
  spec.add_dependency "sample/common", "~> 1.0"
end
"#,
    )
    .expect("write base prayspec");

    fs::write(repo.join("packages/base/README.md"), "package readme\n").expect("write readme");
    fs::write(
        repo.join("packages/base/exports/testing-basics.md"),
        "Testing guidance\n",
    )
    .expect("write export");

    fs::write(
        repo.join("packages/common/sample-common.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/common"
  spec.version = "1.0.0"
  spec.summary = "common guidance"
  spec.files = ["README.md", "exports/common-basics.md"]
  spec.exports = {
    "common-basics" => {
      type: "fragment",
      path: "exports/common-basics.md",
      summary: "Common guidance"
    }
  }
end
"#,
    )
    .expect("write common prayspec");

    fs::write(repo.join("packages/common/README.md"), "package readme\n").expect("write readme");
    fs::write(
        repo.join("packages/common/exports/common-basics.md"),
        "Common guidance\n",
    )
    .expect("write export");
}

pub fn create_derived_fixture(repo: &Path) {
    fs::create_dir_all(repo.join("sample-base/exports")).expect("fixture directories");
    fs::write(
        repo.join("Prayfile"),
        r#"
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
agent "sample/base", "~> 1.4"
render mode: :managed, conflict: :fail, churn: :minimal
"#,
    )
    .expect("write Prayfile");

    fs::write(
        repo.join("sample-base/sample-base.prayspec"),
        r#"
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.4.3"
  spec.summary = "shared guidance"
  spec.files = ["README.md", "exports/testing-basics.md"]
  spec.exports = {
    "testing-basics" => {
      type: "fragment",
      path: "exports/testing-basics.md",
      summary: "Testing guidance"
    }
  }
end
"#,
    )
    .expect("write prayspec");

    fs::write(repo.join("sample-base/README.md"), "package readme\n").expect("write readme");
    fs::write(
        repo.join("sample-base/exports/testing-basics.md"),
        "Testing guidance\n",
    )
    .expect("write export");
}

pub fn read_package_archive(path: &Path) -> BTreeMap<String, String> {
    let file = fs::File::open(path).expect("open package archive");
    let decoder = zstd::stream::read::Decoder::new(file).expect("decode archive");
    let mut archive = tar::Archive::new(decoder);
    let mut entries = BTreeMap::new();
    for entry in archive.entries().expect("read archive entries") {
        let mut entry = entry.expect("archive entry");
        if entry.header().entry_type().is_dir() {
            continue;
        }
        let path = entry
            .path()
            .expect("entry path")
            .to_string_lossy()
            .to_string();
        let mut content = String::new();
        entry.read_to_string(&mut content).expect("entry contents");
        entries.insert(path, content);
    }
    entries
}
