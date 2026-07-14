use jsonschema::Validator;
use pray_core::lockfile::read_lockfile;
use pray_core::manifest::parse_manifest;
use pray_core::package_spec::parse_package_spec;
use pray_core::registry::{RegistryIndex, RegistryPackageMetadata, RegistryPackageVersion};
use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn schema_path(name: &str) -> PathBuf {
    workspace_root().join("schema").join(name)
}

fn load_validator(schema_name: &str) -> Validator {
    let path = schema_path(schema_name);
    let schema_text = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("read schema {}: {error}", path.display());
    });
    let schema_json: serde_json::Value =
        serde_json::from_str(&schema_text).expect("parse schema json");
    Validator::new(&schema_json).expect("compile schema")
}

fn assert_valid(validator: &Validator, value: &serde_json::Value, label: &str) {
    let errors: Vec<String> = validator
        .iter_errors(value)
        .map(|error| error.to_string())
        .collect();
    assert!(
        errors.is_empty(),
        "schema validation failed for {label}: {errors:?}"
    );
}

fn lockfile_to_json(path: &Path) -> serde_json::Value {
    let lockfile = read_lockfile(path).expect("read lockfile");
    serde_json::to_value(lockfile).expect("serialize lockfile to json")
}

#[test]
fn example_lockfiles_validate_against_lockfile_schema() {
    let validator = load_validator("lockfile.schema.json");
    for relative_path in [
        "examples/simple-project/Prayfile.lock",
        "examples/team-workflow/Prayfile.lock",
        "examples/customized-render/Prayfile.lock",
        "Prayfile.lock",
    ] {
        let path = workspace_root().join(relative_path);
        if !path.exists() {
            continue;
        }
        let value = lockfile_to_json(&path);
        assert_valid(&validator, &value, relative_path);
    }
}

#[test]
fn parsed_example_manifests_validate_against_manifest_schema() {
    let validator = load_validator("manifest.schema.json");
    for relative_path in [
        "examples/simple-project/Prayfile",
        "examples/team-workflow/Prayfile",
        "examples/customized-render/Prayfile",
        "Prayfile",
    ] {
        let path = workspace_root().join(relative_path);
        if !path.exists() {
            continue;
        }
        let manifest_text = fs::read_to_string(&path).expect("read prayfile");
        let manifest = parse_manifest(&manifest_text).expect("parse prayfile");
        let value = serde_json::to_value(manifest.canonicalized()).expect("serialize manifest");
        assert_valid(&validator, &value, relative_path);
    }
}

#[test]
fn parsed_example_package_spec_validates_against_package_schema() {
    let validator = load_validator("package.schema.json");
    let path = workspace_root().join("examples/simple-project/packages/base/sample-base.prayspec");
    let package_text = fs::read_to_string(&path).expect("read prayspec");
    let package_spec = parse_package_spec(&package_text).expect("parse prayspec");
    let value = serde_json::to_value(package_spec.canonicalized()).expect("serialize package spec");
    assert_valid(&validator, &value, path.to_str().expect("utf-8 path"));
}

#[test]
fn registry_metadata_validates_against_registry_schema() {
    let validator = load_validator("registry.schema.json");
    let metadata = RegistryPackageMetadata {
        name: "sample/base".to_string(),
        versions: vec![RegistryPackageVersion {
            version: "1.0.0".to_string(),
            artifact: "v1/artifacts/sample/base/1.0.0/package.praypkg".to_string(),
            artifact_hash: Some(
                "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                    .to_string(),
            ),
            tree_hash: Some(
                "sha256:1111111111111111111111111111111111111111111111111111111111111111"
                    .to_string(),
            ),
            yanked: false,
            targets: vec!["tool_a".to_string()],
            exports: vec!["testing-basics".to_string()],
            signer: None,
            signer_fingerprint: None,
            published_at: None,
            signature: None,
            derived_metadata: None,
        }],
    };
    let value = serde_json::to_value(metadata).expect("serialize registry metadata");
    assert_valid(&validator, &value, "registry package metadata");

    let index = RegistryIndex {
        spec: "pray-registry-v1".to_string(),
        packages: vec!["sample/base".to_string()],
    };
    let index_value = serde_json::to_value(index).expect("serialize registry index");
    assert!(
        !validator.is_valid(&index_value),
        "registry package schema should not accept registry index documents"
    );
}
