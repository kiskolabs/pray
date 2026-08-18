## Additional instructions

### .agents/project.md
Repository for the pray open specification and the reference CLI.

Read `README.md` for project positioning and `rfcs/` for the normative Prayfile, prayspec, lockfile, distribution point, and CLI design (RFC 0111).

## Project intent

- Production readiness. Build the reference CLI and specification together, prioritizing validated contracts, user-facing reliability, and test coverage.
- Problem focus. Inference input is operational. Packaging shapes will keep changing. Prayfile targets reproducible composition, provenance markers, and sync of shared input libraries across repositories, not any one vendor workflow.
- What the tool must do. Resolve declared input dependencies, lock exact versions and hashes, render tool-specific files under defined contracts, cite managed blocks with compact pray markers into `Prayfile.lock`, and keep shared input pinned and updatable through manifest and lockfile semantics.
- Production focus. Prefer contract clarity, production validation, and test coverage over premature implementation.

## Rust workspace

For the pray reference implementation, run from the workspace root:

- `cargo test` for the full suite
- `cargo test -p <crate>` for a focused crate
- `cargo clippy` and `cargo fmt --check` before claiming quality checks pass

Use coverage tooling declared in this repository when validating coverage claims.

Prefer files around 150 lines or fewer when cohesion allows. Treat 300 lines as a hard upper bound for any source file unless a very small exception is clearly justified. When a file approaches that ceiling, split by semantic responsibility into separate modules, folders, or helpers rather than by arbitrary line count.

Enforce with `make loc-check` (warn >=150, fail >300 unless ratcheted in `scripts/loc-limits.allowlist`). Language linters mirror the hard ceiling: RuboCop `Pray/FileLength` (Ruby), Biome `noExcessiveLinesPerFile` (TypeScript). Clippy has no file-LOC lint yet.

Test coverage must follow `spec/README.md` guidelines.

## Shared instructions
