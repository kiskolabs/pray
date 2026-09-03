# CHANGELOG

## Unreleased

- Read the whole eight byte tar checksum field so `.praypkg` archives whose checksum is written as seven octal digits unpack instead of failing integrity.

## 1.10.0 (2026-09-02)

- Refuse to overwrite exclusive `file:` and `tree:` destinations that already exist with other bytes.
- Fail when a provisioned or compose destination is a symbolic link.
- Record provisioned leaves in `Prayfile.lock` and delete a dropped leaf only when on-disk bytes still match the locked hash.
- Keep the previous lock when a provisioned destination write fails, so retry still has its ownership record.
- Keep TypeScript lockfiles readable when a generated top-level section is empty.
- Reject destination paths that start with `~`.
- Print every provisioned path in `pray plan`.
- BREAKING: Drop unused `render` fields `section_markers` and `line_endings`; reject them at parse.
- Inline a UTF-8 `file` export into `compose` as a marked span. Exclusive `file:` stays unmarked.
- Write the Agent context banner on `AGENTS.md` by default. Other compose destinations opt in with `header: true`.
- Fail compose of JSON, binary, or an unknown file type and name `file:` as the unmarked path.
- Warn when Prayfile uses deprecated `skills`, export type `skill`, or `spec.skills`; prefer `tree` / `folder`. These forms will be removed in version 2.

## 1.9.2 (2026-09-01)

- Align package version with the 1.9.2 release.
- Reject `.praypkg` members that escape the package root or exceed size limits; unpack through staging into cache.
- Move login sessions to the user Pray home with owner-only permissions and migrate legacy repository sessions.
- Reject manifest and federation paths outside their roots; require registry hashes and recheck cached package trees.
- Bound server requests, connections, timeouts, and federation peers; expose readiness and per-request identifiers.
- Cap registry downloads at 64 MiB; unpack `.praypkg` tar members without system tar; reject absolute artifact URLs and check a present registry signature.
- Enforce a measured line-coverage floor in CI.

## 1.9.1 (2026-08-29)

- Rewrite every matching `pray` line when `pray update --latest` moves a package constraint, keeping indent and extra keywords.
- Preview those constraint moves with `pray update --latest --dry-run` without writing Prayfile.

## 1.7.0 (2026-07-29)

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
