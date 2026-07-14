# Pray serve, install, and confess distribution point flow

## What changed
- Added `pray serve` as a static distribution-point server that serves registry files, package pages, and confession submissions.
- Added registry-based package resolution and install support over HTTP.
- Added `pray confess` to submit accepted/rejected feedback to the active distribution point.
- Extended packaged archives to include the package spec so served artifacts can be resolved by clients.
- Added an end-to-end Ruby/Capybara-style smoke test that installs from a served registry, submits confessions from two client workspaces, and checks the web surface.

## Validation
- `cargo test -p pray --test install serve_install_and_confess_end_to_end_with_web_surface -- --nocapture`
- `cargo test -p pray --test install package_builds_a_tar_zst_archive_from_package_contents -- --nocapture`
- `cargo test`
- `cargo fmt --all --check`
- `cargo clippy --workspace -- -D warnings`
