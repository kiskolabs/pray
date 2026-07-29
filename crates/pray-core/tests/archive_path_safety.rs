use pray_core::paths::validate_package_relative_path;
use std::path::Path;

#[test]
fn rejects_dot_dot_escape_paths() {
    let error =
        validate_package_relative_path(Path::new("exports/../../outside.md")).expect_err("escape");
    assert!(error.to_string().contains("escapes package root"));
}

#[test]
fn rejects_absolute_unix_style_paths() {
    let error = validate_package_relative_path(Path::new("/tmp/owned.md")).expect_err("absolute");
    assert!(error.to_string().contains("must be relative"));
}
