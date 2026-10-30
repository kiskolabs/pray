#[path = "install_support.rs"]
mod support;

use std::fs;

use support::{create_fixture, run_pray, temporary_directory};

#[test]
fn render_check_accepts_unmarked_text_that_install_keeps() {
    let repo = temporary_directory("pray-render-check-unmarked");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    let rendered_path = repo.join("INSTRUCTIONS.md");
    let rendered = fs::read_to_string(&rendered_path).expect("rendered file exists");
    let rendered = rendered.replace(
        "## Additional instructions\n",
        "## Additional instructions\n\nExtra unmarked line\n",
    );
    fs::write(&rendered_path, rendered).expect("rendered file rewritten");
    assert!(run_pray(&repo, &["install"]).status.success());

    let check = run_pray(&repo, &["render", "--check"]);
    assert!(
        check.status.success(),
        "render --check failed:\n{}",
        String::from_utf8_lossy(&check.stderr)
    );
}

#[test]
fn render_check_rejects_an_edited_managed_span() {
    let repo = temporary_directory("pray-render-check-edited");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    let rendered_path = repo.join("INSTRUCTIONS.md");
    let rendered = fs::read_to_string(&rendered_path).expect("rendered file exists");
    let rendered = rendered.replace("Testing guidance\n", "Edited testing guidance\n");
    fs::write(&rendered_path, rendered).expect("rendered file rewritten");

    let check = run_pray(&repo, &["render", "--check"]);
    assert_eq!(check.status.code(), Some(5));
}

#[test]
fn render_check_and_render_preserve_edited_retired_and_unknown_spans() {
    let repo = temporary_directory("pray-render-check-retired");
    create_fixture(&repo);
    assert!(run_pray(&repo, &["install"]).status.success());

    let rendered_path = repo.join("INSTRUCTIONS.md");
    let rendered = fs::read_to_string(&rendered_path).expect("rendered file exists");
    let rendered = rendered.replace("Testing guidance", "Edited retired guidance");
    fs::write(
        &rendered_path,
        format!("{rendered}<!-- pray:unknown1 -->\nUnknown body\n<!-- pray:unknown1 -->\n"),
    )
    .expect("rendered file rewritten");
    let manifest_path = repo.join("Prayfile");
    let manifest = fs::read_to_string(&manifest_path).expect("manifest");
    let manifest = manifest
        .lines()
        .filter(|line| !line.contains("sample/base"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&manifest_path, format!("{manifest}\n")).expect("manifest rewritten");

    assert_eq!(
        run_pray(&repo, &["render", "--check"]).status.code(),
        Some(5)
    );
    let render = run_pray(&repo, &["render"]);
    assert!(
        render.status.success(),
        "render failed: {}",
        String::from_utf8_lossy(&render.stderr)
    );
    let reconciled = fs::read_to_string(&rendered_path).expect("rendered file exists");
    assert!(reconciled.contains("Edited retired guidance"));
    assert!(reconciled.contains("Unknown body"));
    assert!(run_pray(&repo, &["render", "--check"]).status.success());
}
