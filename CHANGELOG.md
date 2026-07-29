# CHANGELOG

## Unreleased

- Publish the Rust CLI as crates.io package `pray-cli` (binary remains `pray`); `pray` is already taken on crates.io.
- Add versioned workspace path dependencies so `pray-core`, `pray-transport`, and `pray-cli` can publish in order.
- Add manual release scripts for crates.io, npmjs, RubyGems, and pray distribution-point publish under `scripts/release/`.
- Point `pray upgrade` and install docs at `cargo install pray-cli`.
- Sync the TypeScript `PACKAGE_VERSION` constant with the npm package version.
- Align crates.io, npm, and RubyGems package descriptions and README openers around Prayfile positioning; drop “reference” from storefront wording.

## 1.5.0 (2026-07-27)

- Accept Gemfile-like Ruby surface sugar for Prayfile statements: `{…}` blocks, top-level `;`, and optional call parentheses on keywords and symbol assignments.
- Align Ruby `pray-cli` with `((pray:…))` symbol maps (`pray do` / `template do`), substitution on render/provision, and the same surface sugar.
- Add TypeScript and Ruby parser/substitute coverage for the surface forms.
- Port `compose` / `tree` / `file:` destination parse, role-based export selection, and scoped render/provision into Ruby `pray-cli`.
- Add a shared Prayfile fixture corpus under `testdata/shared/` exercised by Rust, TypeScript, and Ruby CI suites.
- Port Ruby `pray format` (`fmt`) recommended destination DSL rewrite to match Rust and TypeScript.

## 1.4.0 (2026-07-27)

- Add `((pray:symbol))` templating with a project-wide `pray do … end` symbol map (alias `template do`).
- Substitute symbols in compose fragments, local embeds, and UTF-8 `file:` / tree exports.
- Verify exclusive `file:` bindings against substituted expected content.

## 1.3.0 (2026-07-26)

- Add recommended Prayfile destination forms: `compose`, `tree`, and `pray` with `file:`.
- Select default exports from destination context (fragment, folder or skill, file).
- Add `pray format` (`fmt`) to rewrite Prayfile to the recommended destination DSL.
- Use a single HTTP client for release checks and trust-feed retrieval.
- Allow slim CLI builds to omit local registry authentication storage.

## 1.2.1 (2026-07-25)

- Shrink the release binary with thin LTO, symbol stripping, and a single codegen unit.
- Trim Tokio features and build reqwest with rustls only (no default TLS extras).

## 1.2.0 (2026-07-14)

- Add optional CLI upgrade notice after successful commands when a newer release is available.
- Add `pray upgrade` to install the latest Rust CLI via `cargo install`.
- Point upgrade notice changelog link to the main branch CHANGELOG.

## 1.1.0 (2026-07-14)

- Add environment-aware rendering with `group` blocks and `--env` or `PRAY_ENV`.
- Add global `--path` and `--file-path` flags with `PRAY_PATH`, `PRAY_FILE_PATH`, and project `.env` support.
- Record the selected environment in `Prayfile.lock`.
- Improve CLI help with grouped commands, per-command help, and suggestions for unknown commands.
- Add `--no-input` to skip interactive prompts.
- Honor `PRAY_NO_COLOR` and `NO_COLOR` for plain terminal output.
- Refresh git distribution caches on install when a locked revision is missing locally.

## 1.0.0 (2026-07-13)

- Initial release of the pray reference CLI.
- Resolve local path packages and git distribution sources.
- Publish to local distribution roots and serve over HTTP.
- Install, update, render, verify, and drift workflows with `Prayfile.lock`.
