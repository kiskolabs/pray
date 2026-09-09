use pray_core::{
    render::provisioned_destination_statuses, resolve::resolve_project_in_context,
    resolve_context::ResolveOptions,
};
use std::fs;

#[test]
fn collision_report_is_bounded_and_counts_omissions() {
    let root = std::env::temp_dir().join(format!("pray-destination-budget-{}", std::process::id()));
    fs::create_dir_all(root.join("package/files")).unwrap();
    fs::create_dir_all(root.join("out/files")).unwrap();
    let files: Vec<_> = (0..105)
        .map(|index| format!("files/{index:03}.txt"))
        .collect();
    fs::write(
        root.join("Prayfile"),
        "prayfile \"1\"\ntree \"out\" do\n pray \"sample/files\", path: \"package\"\nend\n",
    )
    .unwrap();
    fs::write(root.join("package/files.prayspec"), format!("Package::Specification.new do |spec|\n spec.name = \"sample/files\"\n spec.version = \"1.0.0\"\n spec.files = {}\n spec.exports = {{ \"files\" => {{ type: \"folder\", path: \"files\" }} }}\nend\n", serde_json::to_string(&files).unwrap())).unwrap();
    for name in files {
        fs::write(root.join("package").join(&name), "package").unwrap();
        fs::write(root.join("out").join(&name), "operator").unwrap();
    }
    let project =
        resolve_project_in_context(&root.join("Prayfile"), &root, &ResolveOptions::default())
            .unwrap();
    let error = provisioned_destination_statuses(&project, None)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("5 additional destination conflicts"),
        "{error}"
    );
    assert!(error.len() < 65_536);
    fs::remove_dir_all(root).unwrap();
}
