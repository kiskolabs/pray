# CHANGELOG

## Unreleased

- Reshape CLI help around a `Usage` synopsis and `Options` block; cover every listed command; point unknown-command errors at `pray --help`.

## 1.6.0 (2026-07-29)

- Add `login` with passkey and ssh-agent modes; persist sessions in `.pray/session.json`.
- Add `upgrade` via `npm install -g pray-cli@latest`.
- Honor `update --major|--latest|--dry-run|--json`, `plan --remote`, and `outdated --remote`.
- Implement `trust import-repo` and `trust import-registry` for local and http(s) sources.

## 1.5.2 (2026-07-29)

- Put the Prayfile positioning line into the npm package description for registry search.

## 1.5.1 (2026-07-29)

- Sync `PACKAGE_VERSION` with the package version and add `prepublishOnly` build for npm publish.
- Rework package description and README opener to match Prayfile positioning.

## 1.5.0 (2026-07-27)

- Accept Gemfile-like Ruby surface sugar for Prayfile statements: `{…}` blocks, top-level `;`, and optional call parentheses.
- Add parser coverage for symbol maps and surface forms alongside substitute tests.
- Exercise the shared `testdata/shared` Prayfile corpus alongside Rust and Ruby.

## 1.3.0 (2026-07-26)

- Add destination DSL parsing: `compose`, `tree`, and `file:` blocks in `Prayfile` for scoped rendering and exact file bindings, matching `pray-core`.
- Add role-based export selection (`fragment`, `folder`, `file`) so packages without an explicit `export:` resolve unambiguous exports automatically.
- Add `format`/`fmt` command to rewrite a legacy `Prayfile` into the recommended destination DSL and normalize marker comments in rendered lockfile outputs.
- Emit `pray` as the canonical package declaration keyword when formatting or editing manifests, while still accepting `use`, `include`, `agent`, and `package` on read for backward compatibility.

## 1.1.0 (2026-07-14)

- Add environment-aware rendering with `group` blocks and `--env` or `PRAY_ENV`.
- Add global `--path` and `--file-path` flags with `PRAY_PATH`, `PRAY_FILE_PATH`, and project `.env` support.
- Record the selected environment in `Prayfile.lock`.
- Improve CLI help with grouped commands, per-command help, and suggestions for unknown commands.
- Add `--no-input` to skip interactive prompts.
- Honor `PRAY_NO_COLOR` and `NO_COLOR` for plain terminal output.
- Refresh git distribution caches on install when a locked revision is missing locally.

## 1.0.0 (2026-07-13)

- Initial npm release of `pray-cli`.
- Resolve local path packages and git distribution sources.
- Publish to local distribution roots and serve over HTTP.
- Install, update, render, verify, and drift workflows with `Prayfile.lock`.
- Git distribution integration tests for install, locked revision, and update.
