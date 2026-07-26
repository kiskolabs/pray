# CHANGELOG

## Unreleased

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
