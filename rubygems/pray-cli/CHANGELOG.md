# CHANGELOG

## Unreleased

- Fix `pray package` and `pray publish` when `tmpdir` is not already loaded.

## 1.18.0 (2026-09-17)

- Clone a git catalog without unused package blobs, and fetch a used package file when install needs it.
- Skip cloning a git catalog when no package uses that source.
- Fetch a used git catalog once on `pray update` instead of fetching it twice.
- Keep packages from two git sources that share a clone URL when each source names a different `subdir`.
- Share one git object store for those subdirectory checkouts.
- Import signing keys with `pray trust import-repo` after a catalog was cloned only through `subdir` sources.
- Reuse registry package metadata during one command so `pray update --latest` does not download the same package JSON twice.
- Skip a second resolve when `pray update --latest` does not rewrite constraints or upstream pins, and still refresh path-fork files when needed.

## 1.17.0 (2026-09-16)

- Locate local prayers through a path source of any directory name (RFC 0117). `pray prayer init` adds `source "local"` when needed and declares `pray "local/project"` once. Version is optional until `pray package` or `pray publish`. `.agents/project.md` remains a compose shortcut for one local file.
- Copy upstream files into an empty path-fork tree on `pray install` and `pray update` (RFC 0116). Extra listed overlay files stay. `pray outdated` lists fork files that differ from the locked upstream. An identity-only fork uses empty `spec.files`; refresh lists content paths, not the prayspec file.
- Refuse a local path with `file:` and a local path inside `tree`. Compose still embeds local fragments. Package `file:` and package `tree` stay.
- Name dest versus lock on `pray verify`, dest versus a fresh render on `pray drift`, and dest write on `pray render`. Dry-run of a write is `pray plan`. Listed local compose files stay on `pray install`.
- Write `.praytorrent.json` when `v1/distribution.json` lists `protocols: ["torrent"]` (RFC 0062). `pray repo init` writes empty protocols.
- Parse `spec.maintainers` as a string array, and accept `spec.pray_version` as an alias for `spec.prayfile_version`.
- Re-embed a changed local compose source on `pray install`, `pray update`, and `pray plan`, and print its `sha256` as `checked` or `was`. List local checksum drift under `pray outdated`.

## 1.16.0 (2026-09-15)

- Rewrite a two-component pessimistic Prayfile pin on `pray update --latest` (`~> 2.2` to `~> 2.4` when registry latest is 2.4.0), matching the Rust and TypeScript CLIs. Install a newer version that the current constraint already allows.
- Resolve a local `.praypkg` from `tarball:`, including offline when the archive is on disk.
- Refuse two-package dependency cycles during resolve, matching the Rust CLI.
- Check dest files against `Prayfile.lock` managed spans without resolving packages (`inspect_locked_destinations`, RFC 0106).

## 1.15.0 (2026-09-15)

- Refresh path-fork trees with `pray update` (RFC 0114). Rewrite `spec.upstream` pins with `pray update --latest` when the pin does not admit the latest upstream version. Rewrite Prayfile constraints with `pray update --latest` when they do not admit the registry latest version. `pray update --latest --dry-run` prints the planned rewrite and does not write.
- Keep `spec.upstream` in the packaged prayspec. Published registry metadata does not copy that pin.
- Keep a listed package spec in `spec.files` when packing; refuse a repeated or aliased content path in the package archive.
- Use the current project for a later install after an earlier in-process command in another directory.

## 1.14.0 (2026-09-14)

- Keep an unchanged package version's artifact, first-publish time, and yank when `pray publish` runs again (RFC 0061).
- Write `published_at` as UTC Unix seconds in registry and federation JSON.
- Collapse trailing blank lines when composing so a composed file ends with a single newline, matching the other CLIs.
- Order `symbols` before `render` in the canonical manifest JSON so `manifest_hash` matches the other CLIs.

## 1.13.0 (2026-09-08)

- Record and verify package upstream pins in `.prayspec` and `Prayfile.lock`. Refuse path-fork updates until this CLI can refresh the path tree safely (RFC 0114).

## 1.12.1 (2026-09-08)

- Refresh a locked git catalog during `pray install` when a newly declared package is missing from the pinned revision, and name that revision with `pray update` if it is still missing.

## 1.12.0 (2026-09-07)

- Speed up planning for large package trees and reuse the resolved lock during installation.
- Report duplicate lockfile fields as parsing errors with lockfile context.
- Restore compose files, provisioned destinations, Prayfile, and lock after failed writes, and recover interrupted writes on the next install, plan, or verification on Unix.
- Preserve later local edits when recovery encounters changed files, and prevent cooperating Pray commands from writing the same project together.
- Limit destination reads to 32 MiB, saved transaction payload to 64 MiB across 10,000 writes, and grouped conflict details to 100 entries.
- Report conflicting file and tree destinations together before changing compose output, with steps that preserve local edits.
- Reject unsupported update options (`--latest`, `--major`, `--dry-run`, and `--json`) instead of silently ignoring them.

## 1.11.0 (2026-09-04)

- Read the whole eight byte tar checksum field so `.praypkg` archives whose checksum is written as seven octal digits unpack instead of failing integrity.
- Share the source-keyed registry cache path with the Rust and TypeScript CLIs.
- Add `pray clean --unused` for lockfile-driven registry cache cleanup.
- Reject unsafe registry package and version path segments.

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
