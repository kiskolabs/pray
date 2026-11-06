# Dependency optimization audit (drop, thin, or custom)

## Decisions

Run an iterative engineering audit of direct dependencies across Rust, npm, and Ruby. Rank candidates by hot-path weight and replaceability. Prove each change with tests that fail on a missing contract or preserve observed display and lint behavior before claiming a drop. Do not replace cryptography, HTTP, archive codecs, TOML, or semver with custom code in this pass.

Accepted in this session after tests:

- Drop `anyhow` from `pray-transport` (`TransportError::Other` is now `String`).
- Drop `thiserror` from `pray-transport` (hand-written `Display` / `Error` / `From`).
- Split `TransportError` into `crates/pray-transport/src/error.rs` so file LOC stays under 300.
- Replace `madge` with `npmjs/scripts/check-import-cycles.mjs` for `lint:cycles`, with fixture and parity tests.
- Remove the Dependabot TypeScript major ignore that existed only for the madge peerOptional constraint.

Keep for now (evidence below): `async-trait`, `reqwest`, `tokio`, `serde`/`serde_json`, `toml`/`smol-toml`/`perfect_toml`, `semver`, `sha2`, `ed25519-dalek`, `getrandom`, `base64`, `tar`, `zstd`, `rusqlite` (optional auth), `libc`, `jsonschema` (dev schema contract), Ruby `base64` default gem pin.

## Effects

Inventory (observed):

- Rust direct production deps center on parse/lock (`toml`, `serde`), constraint (`semver`), integrity (`sha2`, `ed25519-dalek`, `base64`, `getrandom`), packages (`tar`, `zstd`), network (`reqwest`, `tokio`), auth store (`rusqlite` optional), and unix flags (`libc`).
- `pray-transport` previously pulled `anyhow` and `thiserror` only for `TransportError`.
- `TransportAdapter` is stored as `Box<dyn TransportAdapter>` with async methods, so `async-trait` remains required for object safety without a factory redesign.
- `base64` is also transitive via `reqwest`; removing the direct dep would not remove it from the graph.
- npm production deps are `semver` and `smol-toml`. `madge` was lint-only and pulled the minimatch advisory chain.
- Ruby runtime deps are `perfect_toml` and `base64` (stdlib/default gem).

Spike 1, drop anyhow:

- Added display and `From<String>` tests for `TransportError::Other`.
- Replaced `Other(anyhow::Error)` with `Other(String)` and updated http/ssh call sites.
- `cargo test -p pray-transport` PASS (16 then 17 tests across spikes).
- `cargo tree -p pray-transport -i anyhow` reports no match; workspace lock dropped the unused `anyhow` package.

Spike 2, drop thiserror:

- Hand-wrote `Display`, `Error::source`, and `From` for io/json/string.
- Added variant display-prefix tests.
- Split `error.rs` (113 LOC) and `types.rs` (264 LOC).
- `cargo test -p pray-transport` PASS; `cargo clippy -p pray-transport --all-targets --all-features -- -D warnings` PASS; `cargo check -p pray-cli` PASS.
- Direct `thiserror` removed from `pray-transport`; may remain transitive elsewhere in the workspace graph.

Spike 3, replace madge:

- `node --test npmjs/scripts/check-import-cycles.test.mjs` PASS (synthetic cycle detected; pray-core and pray-cli entrypoints empty, matching madge).
- CLI checker reports no cycles on the same entrypoints madge used.
- After removing madge: `npm install` removed 135 packages; `npm audit` and `npm audit --omit=dev` both report 0 vulnerabilities.
- `npm run lint:cycles` in both workspaces PASS.

Resource note for madge drop (observed): development `node_modules` package count fell from about 200 audited packages to 65. No claim that runtime RSS or shipped npm archive changed; production audit was already clean.

## Next

- Optional follow-up spike: TypeScript 6/7 major now that madge no longer pins the peer; require full npm test/lint/runtime matrix before landing.
- Optional follow-up: redesign `TransportAdapter` away from `dyn` + async (enum dispatch or boxed futures) if dropping `async-trait` is worth the churn; not proven in this pass.
- Optional follow-up: measure release binary size before/after the transport error changes with identical `cargo build --release -p pray-cli` probes; expected delta is small versus archive/TLS crates.
- Do not custom-implement `semver`, TOML, zstd/tar, or ed25519 without a separate RFC-level contract suite.
- Update or supersede `docs/dependencies/20261102115928_npm-minimatch-development-graph-advisory.md` status to fixed by madge removal when this branch lands.

## Source

Commands and evidence from this session and worktrees validating transport and npm changes. Related notes: `docs/issues/20261106093128_rust-cargo-upgrade-qa.md`, `docs/issues/20261106094500_dependency-upgrade-qa.md`, `docs/dependencies/20261102115928_npm-minimatch-development-graph-advisory.md`, `docs/dependencies/20260829222001_madge-typescript-peer.md`.
