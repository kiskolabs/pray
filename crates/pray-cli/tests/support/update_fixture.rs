use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub struct UpdateFixture {
    pub root: PathBuf,
    pub consumer: PathBuf,
}

impl UpdateFixture {
    pub fn new(constraint: &str) -> Self {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("pray-update-{stamp}"));
        let consumer = root.join("consumer");
        for directory in ["source/package/exports", "distribution", "consumer/rules"] {
            fs::create_dir_all(root.join(directory)).unwrap();
        }
        let fixture = Self { root, consumer };
        fs::write(
            fixture.root.join("source/Prayfile"),
            "prayfile \"1\"\npray \"sample/files\", \">= 1.0\", path: \"package\"\n",
        )
        .unwrap();
        fixture.git(&["init", "-b", "main"]);
        fixture.git(&["config", "user.name", "Pray Test"]);
        fixture.git(&["config", "user.email", "pray@example.invalid"]);
        fixture.publish("1.0.0");
        fs::write(
            fixture.consumer.join("rules/rules.prayspec"),
            r#"
Package::Specification.new do |spec|
  spec.name = "sample/rules"
  spec.version = "1.0.0"
  spec.files = ["rules.md"]
  spec.exports = { "rules" => { type: "fragment", path: "rules.md" } }
end
"#,
        )
        .unwrap();
        fs::write(fixture.consumer.join("rules/rules.md"), "old rules\n").unwrap();
        fs::write(
            fixture.consumer.join("Prayfile"),
            format!(
                r#"
prayfile "1"
source "dist", "git+file://{}"
tree "skills" do
  pray "sample/files", "{constraint}", source: "dist"
end
compose "INSTRUCTIONS.md" do
  pray "sample/rules", "~> 1.0", path: "rules"
end
"#,
                fixture.root.join("distribution").display()
            ),
        )
        .unwrap();
        fixture.success(&["install"]);
        fixture
    }

    pub fn publish(&self, version: &str) {
        fs::write(
            self.root.join("source/package/files.prayspec"),
            format!(
                r#"
Package::Specification.new do |spec|
  spec.name = "sample/files"
  spec.version = "{version}"
  spec.summary = "fixture"
  spec.files = ["exports/a.md", "exports/b.md"]
  spec.exports = {{ "files" => {{ type: "folder", path: "exports" }} }}
end
"#
            ),
        )
        .unwrap();
        for name in ["a.md", "b.md"] {
            fs::write(self.root.join("source/package/exports").join(name), version).unwrap();
        }
        let result = self.run_at(
            &self.root.join("source"),
            &[
                "publish",
                "--root",
                self.root.join("distribution/prayers").to_str().unwrap(),
            ],
        );
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        self.git(&["add", "-A"]);
        self.git(&["commit", "-m", version]);
    }

    pub fn omit_ledger(&self) {
        let path = self.consumer.join("Prayfile.lock");
        let mut lockfile = pray_core::lockfile::read_lockfile(&path).unwrap();
        lockfile.provisioned.clear();
        lockfile.generated_by = "pray 1.9.1".into();
        pray_core::lockfile::write_lockfile(&path, &lockfile).unwrap();
    }

    pub fn run(&self, arguments: &[&str]) -> Output {
        self.run_at(&self.consumer, arguments)
    }

    pub fn success(&self, arguments: &[&str]) -> Output {
        let output = self.run(arguments);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }

    fn run_at(&self, directory: &Path, arguments: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_pray"))
            .args(arguments)
            .current_dir(directory)
            .env_remove("PRAY_PATH")
            .env_remove("PRAY_FILE_PATH")
            .env_remove("PRAY_ENV")
            .env("PRAY_HOME", self.root.join("home"))
            .env("PRAY_CACHE", self.root.join("cache"))
            .env("PRAY_TRUST_ASSUME_YES", "1")
            .output()
            .unwrap()
    }

    fn git(&self, arguments: &[&str]) {
        let output = Command::new("git")
            .args([
                "-c",
                "commit.gpgsign=false",
                "-c",
                "core.hooksPath=/dev/null",
            ])
            .args(arguments)
            .current_dir(self.root.join("distribution"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

impl Drop for UpdateFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
