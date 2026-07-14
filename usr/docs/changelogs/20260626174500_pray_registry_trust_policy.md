# Pray registry trust policy fixture

## What changed
- Added a registry trust settings model with email confirmation modes plus passkey, SSH key, and SSH-agent capability flags.
- Added `v1/trust.json` support and surfaced the trust policy on the distribution-point root page.
- Added a focused web-surface fixture that proves the served registry reads and renders trust policy settings.

## Validation
- `cargo test -p pray --test trust serves_registry_trust_policy_on_the_root_page -- --nocapture`
- `cargo test`
- `cargo fmt --all --check`
- `cargo clippy --workspace -- -D warnings`
