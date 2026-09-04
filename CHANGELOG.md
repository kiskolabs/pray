# CHANGELOG

## Unreleased

## 1.11.0 (2026-09-04)

- Read the whole eight byte tar checksum field so `.praypkg` archives whose checksum is written as seven octal digits unpack instead of failing integrity.
- Sort tree hash entries by UTF-8 bytes in the TypeScript CLI so a package hashes the same as where it was published, instead of following the host's locale collation.
- Use the same source-keyed `.pray/cache/registry/<namespace>/<name>/<version>/<source-hash>` path in the Rust, Ruby, and TypeScript CLIs.
- Add `pray clean --unused` to remove registry cache entries absent from the current lockfile while preserving Git cache, vendor, and project state.
- Reject unsafe registry package and version path segments before writing cache entries.

## 1.10.0 (2026-09-02)

- Refuse to overwrite exclusive `file:` and `tree:` destinations that already exist with other bytes.
- Fail when a provisioned or compose destination is a symbolic link.
- Record provisioned leaves in `Prayfile.lock` and delete a dropped leaf only when on-disk bytes still match the locked hash.
- Keep the previous lock when a provisioned destination write fails, so retry still has its ownership record.
- Reject unsafe provisioned paths from a lock before pruning and make `pray plan` refuse the same unmanaged destinations as apply.
- Keep TypeScript lockfiles readable when a generated top-level section is empty.
- Reject destination paths that start with `~`.
- Print every provisioned path in `pray plan`.
- Document home-as-root as `pray --path "$HOME"` with `file: ".zshrc"`, HTML-only compose, and unmarked `file:`.
- BREAKING: Drop unused `render` fields `section_markers` and `line_endings`; reject them at parse.
- Rewrite `Prayfile.lock` `manifest_hash` on install after those unused fields leave the canonical manifest.
- Inline a UTF-8 `file` export into `compose` as a marked span. Exclusive `file:` stays unmarked.
- Write the Agent context banner on `AGENTS.md` by default. Other compose destinations opt in with `header: true`.
- Fail compose of JSON, binary, or an unknown file type and name `file:` as the unmarked path.
- Warn when Prayfile uses deprecated `skills`, export type `skill`, or `spec.skills`; prefer `tree` / `folder`. These forms will be removed in version 2.

## 1.9.2 (2026-09-01)

- Fix Ruby `pray install` unpack of `.praypkg` archives from git and registry sources.
- Treat empty or corrupt registry cache directories as missing so the next install unpacks again.
- Reject `.praypkg` members that escape the package root or exceed size limits on Ruby and TypeScript CLIs.
- Install registry packages through a staging directory, then rename into cache.
- Resolve packages from a matching source namespace or sole source without an explicit `source:` on Ruby `pray-cli`.
- Keep verification secrets out of public registration responses and require a trusted workflow for session and key enrollment.
- Deliver email verification codes to an owner-only file under the registry `.pray` directory; successful verify returns a bearer session.
- Enroll passkeys and SSH keys only with that bearer token. Email-only session creation stays closed.
- Generate authentication secrets from the operating system, store bearer-token hashes, and reject expired credentials.
- Move login sessions from repositories to the user Pray home with owner-only permissions and automatic migration.
- Reject manifest and federation paths outside their roots, require registry integrity hashes, and recheck cached package trees.
- Bound server requests, connections, timeouts, federation peers, and archive expansion; expose server readiness and request identifiers.
- Cap TypeScript and Ruby registry downloads at the same 64 MiB response ceiling as Rust.
- Store hashed email verification secrets, hide why verify failed, and stop after a small number of guesses.
- Unpack `.praypkg` tar members in the Ruby and TypeScript CLIs without handing the archive to system tar.
- Reject absolute artifact URLs; check a present registry signature on TypeScript install.
- Gate Rust, TypeScript, and Ruby line coverage and make the Rust security-boundary mutation smoke blocking.

## 1.9.1 (2026-08-29)

- Rewrite every matching `pray` line when `pray update --latest` moves a package constraint, keeping indent and extra keywords.
- Preview those constraint moves with `pray update --latest --dry-run` without writing Prayfile.
- Record lock marker positions from the file `pray install` wrote so `pray verify` matches after local unmarked text or marker order differs.
- Keep `pray install --locked` on a fresh compose so a changed local compose source still needs an install.

## 1.9.0 (2026-08-18)

- BREAKING: TypeScript and Ruby `pray-cli` reject render conflict values other than `fail`. Manifest schema enums the same.
- Document how Prayfile sits beside Git, Mercurial, Subversion, and CVS.
- Retire `SPEC.md`. Numbered RFCs under `rfcs/` are the product contract (RFC 0111).
- Add RFC process (RFC 0001), template, and id claims in `rfcs/ids/NNNN`.
- Record shipped design as Stable Standards Track RFCs 0010, 0011, 0020, 0030, 0031, 0040, 0050, and 0060.
- Record Informational Stable RFCs 0002, 0070, and 0101.
- Record Experimental follow-ons RFC 0100, 0102, 0104, and 0108.

## 1.8.1 (2026-07-29)

- Group `position_drift` findings per target in `pray verify`, `pray drift`, and install warnings.
- Cite lock versus file marker lines and a `path:line` cause when unmarked preamble differs from compose sources.
- Match the same grouped drift report in TypeScript and Ruby `pray-cli`.

## 1.8.0 (2026-07-29)

- Add `pray yank` / `--undo` to mark yanked versions in a distribution root (metadata only).
- Keep locked yanked versions with a warning; `pray install --strict` refuses them; new resolves and `pray update` skip yanked versions.
- Add HTML-free packaging smoke: publish `--root` → install → verify → yank.
- Document static distribution discovery and CI `--root` publish in `docs/static-distribution.md`.
- Add scoped publish tokens (`pray token create|revoke`, `PRAY_PUBLISH_TOKEN`) for `pray publish --server`.
- Add `pray trust set-require-signed-packages` to refuse unsigned remote packages under a source prefix.
- Add `pray search` for substring matches over a distribution index (no ranking).

## 1.7.0 (2026-07-29)

- Reshape CLI help around a `Usage` synopsis and `Options` block; drop documentation URL and exit-code footers from default help.
- Add per-command help for every listed command so `pray help <command>` no longer reports known commands as unknown.
- Point unknown-command errors at `pray --help`.
- Add `pray completion bash|zsh|fish` for shell completion scripts.
- Document CLI exit codes in `docs/cli-exit-codes.md` and SPEC section 66; add man page `docs/man/pray.1`.

## 1.6.0 (2026-07-29)

- Warn when Prayfile still uses deprecated `target`, `output`, or `agent`; prefer `compose` / `tree` / `pray`. These forms will be removed in version 2.
- Reject package dependency cycles during resolve with a clear resolution error.
- Add parser property tests (`proptest`) and a local `cargo-fuzz` harness for Prayfile, prayspec, and package path validation.
- Harden `.praypkg` unpack path-escape coverage and grow the shared/conformance fixture trees.
- Add CLI exit-code, signature negative-path, and network unreachable failure tests; soft Rust coverage floor in CI.

## 1.5.2 (2026-07-29)

- Put the Prayfile positioning line into crates.io, npm, and RubyGems package descriptions so registry search finds “language before inference”.

## 1.5.1 (2026-07-29)

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
