use pray_core::registry::{
    select_package_version_for_test, set_package_version_yanked, RegistryPackageMetadata,
    RegistryPackageVersion,
};
use pray_core::registry_select::apply_yank_policy;

fn version(version: &str, yanked: bool) -> RegistryPackageVersion {
    RegistryPackageVersion {
        version: version.to_string(),
        artifact: format!("v1/artifacts/sample/base/{version}/package.praypkg"),
        artifact_hash: Some(format!("sha256:{version}")),
        tree_hash: Some(format!("sha256:tree-{version}")),
        yanked,
        ..RegistryPackageVersion::default()
    }
}

fn metadata(versions: Vec<RegistryPackageVersion>) -> RegistryPackageMetadata {
    RegistryPackageMetadata {
        name: "sample/base".to_string(),
        versions,
    }
}

#[test]
fn new_resolution_skips_yanked_versions() {
    let metadata = metadata(vec![version("1.0.0", true), version("1.1.0", false)]);
    let selected = select_package_version_for_test(&metadata, ">= 1.0.0", None)
        .expect("should select non-yanked");
    assert_eq!(selected.version, "1.1.0");
    assert!(!selected.yanked);
}

#[test]
fn new_resolution_fails_when_only_yanked_versions_match() {
    let metadata = metadata(vec![version("1.0.0", true), version("1.1.0", true)]);
    let error = select_package_version_for_test(&metadata, ">= 1.0.0", None)
        .expect_err("only yanked versions must not resolve");
    assert!(
        error.to_string().contains("no registry version"),
        "unexpected error: {error}"
    );
}

#[test]
fn locked_preferred_version_may_remain_yanked() {
    let metadata = metadata(vec![version("1.0.0", true), version("1.1.0", false)]);
    let selected = select_package_version_for_test(&metadata, ">= 1.0.0", Some("1.0.0"))
        .expect("locked yanked version should continue");
    assert_eq!(selected.version, "1.0.0");
    assert!(selected.yanked);
}

#[test]
fn update_style_resolution_without_preferred_leaves_yanked() {
    let metadata = metadata(vec![version("1.0.0", true), version("1.1.0", false)]);
    let selected = select_package_version_for_test(&metadata, ">= 1.0.0", None)
        .expect("update should prefer non-yanked");
    assert_eq!(selected.version, "1.1.0");
}

#[test]
fn yank_policy_warns_by_default_and_fails_when_strict() {
    let yanked = version("1.0.0", true);
    apply_yank_policy("sample/base", &yanked, false).expect("default allows locked yanked");
    let error =
        apply_yank_policy("sample/base", &yanked, true).expect_err("strict must refuse yanked");
    assert!(
        error.to_string().contains("yanked"),
        "unexpected error: {error}"
    );
    apply_yank_policy("sample/base", &version("1.1.0", false), true)
        .expect("non-yanked ok under strict");
}

#[test]
fn set_package_version_yanked_flips_flag_only() {
    let mut metadata = metadata(vec![version("1.0.0", false), version("1.1.0", false)]);
    set_package_version_yanked(&mut metadata, "1.0.0", true).expect("yank");
    assert!(metadata.versions[0].yanked);
    assert!(!metadata.versions[1].yanked);
    assert_eq!(
        metadata.versions[0].artifact_hash.as_deref(),
        Some("sha256:1.0.0")
    );
    set_package_version_yanked(&mut metadata, "1.0.0", false).expect("unyank");
    assert!(!metadata.versions[0].yanked);
}
