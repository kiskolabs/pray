# Pray publish signing and distribution-point verification

## What changed
- Added `pray publish --root PATH` to package project inputs and upload signed package archives and registry metadata into a static distribution-point tree.
- Added registry metadata signer, signature, and published-at fields, plus artifact-hash verification during install.
- Extended the served package web page to show signer and signature details when present.
- Updated the Ruby/Capybara-style smoke test to verify the published registry through two clients and assert the signed web surface.

## Validation
- `cargo test -p pray --test install publish_serve_install_and_confess_end_to_end_with_web_surface -- --nocapture`
- `cargo test`
- `cargo fmt --all --check`
- `cargo clippy --workspace -- -D warnings`
