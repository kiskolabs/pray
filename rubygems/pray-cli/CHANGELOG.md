# CHANGELOG

## Unreleased

- Read the whole eight byte tar checksum field so `.praypkg` archives whose checksum is written as seven octal digits unpack instead of failing integrity.

## 1.10.0 (2026-09-02)

- Refuse to overwrite exclusive `file:` and `tree:` destinations that already exist with other bytes.
- Fail when a provisioned or compose destination is a symbolic link.
- Record provisioned leaves in `Prayfile.lock` and delete a dropped leaf only when on-disk bytes still match the locked hash.
- Keep the previous lock when a provisioned destination write fails, so retry still has its ownership record.
- Reject destination paths that start with `~`.
- Print every provisioned path in `pray plan`.
- BREAKING: Drop unused `render` fields `section_markers` and `line_endings`; reject them at parse.
- Inline a UTF-8 `file` export into `compose` as a marked span. Exclusive `file:` stays unmarked.
- Write the Agent context banner on `AGENTS.md` by default. Other compose destinations opt in with `header: true`.
- Fail compose of JSON, binary, or an unknown file type and name `file:` as the unmarked path.
- Warn when Prayfile uses deprecated `skills`, export type `skill`, or `spec.skills`; prefer `tree` / `folder`. These forms will be removed in version 2.

## 1.9.2 (2026-09-01)

- Fix `.praypkg` unpack for git and registry installs when the Ruby process forces UTF-8 internal encoding.
- Treat empty or corrupt `.pray/cache/registry` directories as not ready so the next install unpacks again.
- Reject archive members that escape the package root or exceed size limits; unpack through staging into cache.
- Resolve packages from a matching source namespace or sole source without an explicit `source:`.
- Move login sessions to the user Pray home with owner-only permissions and migrate legacy repository sessions.
- Reject manifest paths outside the project; require registry hashes and recheck cached package trees.
- Bound server requests, headers, connections, and timeouts; expose a readiness endpoint.
- Cap registry downloads at 64 MiB; unpack `.praypkg` tar members without system tar; reject absolute artifact URLs.

## 1.9.1 (2026-08-29)

- Rewrite every matching `pray` line when a package constraint is updated, keeping indent and extra keywords.

## 1.7.0 (2026-07-29)

- Reshape CLI help around a `Usage` synopsis and `Options` block; cover every listed command; point unknown-command errors at `pray --help`.

## 1.6.0 (2026-07-29)

- Run tests with polyrun parallel RSpec, coverage gate, Makefile lint/test, and RBS validate.
- Implement HTTP `login` (passkey and ssh-agent), `confess`, and `sync`.
- Implement full `trust` CLI against `trust.toml` (list/show/add-key/remove-key/set-*/import-*/check).
- Serve federation discovery, sync index/package, and confession submit over HTTP.

## 1.5.2 (2026-07-29)

- Put the Prayfile positioning line into the gem summary for RubyGems search.

## 1.5.1 (2026-07-29)

- Add `rake build` / `rake release` for gem packaging and RubyGems push.
- Rework gem summary, description, and README opener to match Prayfile positioning.

## 1.5.0 (2026-07-27)

- Add `((pray:symbol))` templating with project-wide `pray do` / `template do` symbol maps.
- Substitute symbols in rendered fragments, local embeds, and UTF-8 provisioned files.
- Accept Gemfile-like Ruby surface sugar: `{…}` blocks, top-level `;`, and optional call parentheses.
- Accept `pray` / `use` / `include` / `package` as package declaration aliases of `agent`.
- Parse and apply `compose`, `tree`, and `file:` destinations with role-based export selection and scoped render/provision.
- Exercise the shared `testdata/shared` Prayfile corpus alongside Rust and TypeScript.
- Rewrite Prayfile with `pray format` (`fmt`) to the recommended compose/tree/file destination DSL.

## 1.1.0 (2026-07-14)

- Add environment-aware rendering with `group` blocks and `--env` or `PRAY_ENV`.
- Add global `--path` and `--file-path` flags with `PRAY_PATH`, `PRAY_FILE_PATH`, and project `.env` support.
- Record the selected environment in `Prayfile.lock`.
- Improve CLI help with grouped commands, per-command help, and suggestions for unknown commands.
- Add `--no-input` to skip interactive prompts.
- Honor `PRAY_NO_COLOR` and `NO_COLOR` for plain terminal output.
- Refresh git distribution caches on install when a locked revision is missing locally.

## 1.0.0 (2026-07-13)

- Initial RubyGems release of `pray-cli`.
- Resolve local path packages and git distribution sources.
- Publish to local distribution roots and serve over HTTP.
- Install, update, render, verify, and drift workflows with `Prayfile.lock`.
- Git distribution integration tests for install, locked revision, and update.
