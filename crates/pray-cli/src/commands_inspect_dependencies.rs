use pray_core::package_spec::PackageSpec;

pub(crate) fn package_dependency_lines(package: &PackageSpec) -> [String; 2] {
    let required = package
        .dependencies
        .iter()
        .filter(|dependency| !dependency.optional)
        .map(|dependency| dependency.name.clone())
        .collect::<Vec<_>>();
    let optional = package
        .dependencies
        .iter()
        .filter(|dependency| dependency.optional)
        .map(|dependency| format!("{} ({})", dependency.name, dependency.constraint))
        .collect::<Vec<_>>();
    [
        format!("dependencies: {}", format_list(&required)),
        format!("optional referrals: {}", format_list(&optional)),
    ]
}

fn format_list(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}
