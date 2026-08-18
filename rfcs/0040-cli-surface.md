# RFC 0040: CLI surface

- Feature Name: cli-surface
- Type: Standards Track
- Status: Stable
- Describes: 1.8.1
- Created: 2026-08-17
- Author: Andrei Makarov
- Relates: RFC 0070, RFC 0100

## Summary

This RFC inventories the `pray` command set as implemented in the Rust reference CLI, and notes the Ruby and TypeScript ports. The registrar list is the contract.

## Motivation

Operators and CI scripts need a stable verb list and exit codes.

## Guide-level explanation

Core loop: `init` / `add` / `install` / `plan` / `apply` / `verify` / `drift`. Package loop: `package` / `publish` / `yank` / `search` / `serve`. Identity: `login`, `token`, `trust`, `confess`. Hygiene: `format`, `vendor`, `clean`, `tree`, `list`, `outdated`, `explain`.

`prayer init` and `repo init` scaffold a package tree and a distribution root.

Unknown commands MUST fail toward `pray --help` (exit 2). `pray help <command>` MUST document every listed command.

## Reference-level explanation

This section is the product contract for this concern. Where it disagrees with Implementation notes, Implementation notes record what the reference CLI does today. A follow-on RFC records the gap.

The registrar verb list in this RFC is the contract. The former snapshot command list omitted unlock, upgrade, login, token, search, yank, trust, sync, and completion; those verbs remain.

Exit codes from `PrayError::exit_code`:

- 0 success
- 1 general error (I/O, missing or invalid manifest context); also `Io` and `Manifest`
- 2 parse error or usage/CLI argument error
- 3 resolution error
- 4 integrity error
- 5 render/check failed
- 6 verify failed (also when `pray drift` finds drift)
- 7 network/fetch error
- 8 unsupported feature

### 50. CLI command set

Primary: `init`, `add`, `install`, `update`, `plan`, `apply`, `render`, `format`/`fmt`, `verify`, `drift`, `package`, `publish`, `yank`, `confess`, `serve`, `vendor`, `clean`.

Also useful: `remove`, `unlock`, `tree`, `list`, `outdated`, `explain`, `manifest`, `search`, `login`, `token`, `trust`, `sync`, `upgrade`, `completion`, `version`, `prayer init`, `repo init`.

---

### 51. CLI command semantics

- init: minimal Prayfile; optional `--targets tool_a,tool_b`
- add / remove: edit package declaration; remove re-renders
- install: from Prayfile and Prayfile.lock
- update: re-resolve versions within constraints
- plan: compute lock/cache/render changes
- apply: materialize plan; refresh managed span checksums and lines
- render: regenerate or check targets; does not replace apply for lock refresh unless documented
- format / fmt: rewrite Prayfile to `compose` / `tree` / `pray …, file:`; normalize markers in lock targets
- verify: read-only managed-span, integrity, cache, signature checks
- drift: custom implementation, removed prayers, position/renderer drift, orphan markers
- package / publish: build `.praypkg`; sign and upload
- confess: signed acceptance/rejection feedback
- serve: local or self-hosted distribution point
- yank: mark a version yanked in a distribution root; `--undo` clears the flag
- vendor / clean: copy into `.pray/vendor`; remove cache/ephemeral state
- tree: dependency graph
- unlock: drop the lock pin for one package
- search: substring match over a distribution index
- login / token: registry identity and scoped publish tokens
- trust: signature policy, including `set-require-signed-packages`
- sync: record publish or sync updates through a VCS backend
- upgrade: install the latest Rust CLI via `cargo install`
- completion: emit bash, zsh, or fish completion
- version: print CLI version
- list / outdated / explain: inventory locked packages

---

### 61. Platform behavior

Must work on common desktop and server platforms where practical

Rules:

- files are UTF-8
- generated text uses LF by default
- paths in manifest and lock use `/`
- implementation converts at filesystem boundary
- no absolute paths in lockfile
- no platform-specific resolution unless declared
- cache path follows OS conventions
- target output paths are repository-relative unless local mode is used

---

### 62. Tool discovery

May detect installed tools, but discovery must not change lockfile resolution unless explicitly requested. Warn on missing tools during verify; never silently change the lock because a tool is present locally. Resolution is manifest-driven.

---

### 63. Formatting policy

Formatting is not the product.

Still, a formatter exists: `pray format` (alias `pray fmt`)

`pray format` rewrites Prayfile to the recommended destination DSL:

- `compose "path" do … end` for fragment outputs and local embeds
- `tree "path" do … end` for folder/skill roots
- `pray "owner/name", …, file: "path"` for whole-file exports
- `pray` instead of legacy `agent` / `package`
- drop empty legacy `target` wrappers that only declared `output` / `skills`
- keep `target` blocks that still carry `commands`, `rules`, or `max_bytes`
- keep `group` membership via `group` blocks

Legacy migration classifies packages from resolved export kinds (fragment → compose, folder/skill → tree, file → `file:` using `default_path` when present). Format may resolve packages (offline first) to classify.

Rules:

- prefer semantic stability over comment preservation when migrating shapes
- use stable indentation (2 spaces inside blocks)
- prefer one package declaration per dependency in destination blocks
- avoid rewriting whole Prayfile for add/remove
- append new packages at logical location
- make lockfile canonical
- formatting twice should be idempotent

Less churn is more important than stylistic perfection for add/remove; `format` / `fmt` is the explicit opt-in rewrite.

---

### 64. Global config

Optional user config: `~/.config/pray/config.toml`

May contain:

```toml
[cache]
directory = "~/.cache/pray"

[network]
offline = false

[registry.default]
url = "https://agents.example.com"
```

Global config must not inject packages into a repository. The repository dependency graph must come from Prayfile.

---

### 65. Environment variables

Allowed implementation variables:

```
PRAY_HOME
PRAY_CACHE
PRAY_CONFIG
PRAY_NO_COLOR
PRAY_OFFLINE
PRAY_PATH
PRAY_FILE_PATH
PRAY_ENV
```

`PRAY_PATH`, `PRAY_FILE_PATH`, and `PRAY_ENV` select the project root, manifest path, and render environment. Equivalent CLI flags are `--path`, `--file-path`, and `--env` / `--environment`. Precedence is CLI option, process environment, project `.env`, then defaults.

The reference CLI loads one `.env` file from the selected project root hint and reads only `PRAY_PATH`, `PRAY_FILE_PATH`, and `PRAY_ENV` from it without overriding values already set in the process environment.

`--path` owns the project root, `Prayfile.lock`, and rendered outputs. A relative `--file-path` resolves under that root. For `pray add`, place the global project `--path` before the subcommand; `add --path PACKAGE_PATH` remains the package source path.

`PRAY_ENV` and its CLI equivalents affect rendering and provisioning only. They must not change which packages are resolved or locked.

They may affect local behavior. They must not silently change package resolution.

---

### 66. Error style

Errors must be precise.

Good:

```
Prayfile:14: package "sample/webapp" requests export "testing2", but available exports are: "testing", "data-layer", "webapp-review".
```

Bad:

```
Resolution failed.
```

Error categories: `parse_error`, `manifest_error`, `resolution_error`, `fetch_error`, `integrity_error`, `render_error`, `verify_error`, `target_error`

Exit codes (reference CLI):

- 0: success
- 1: general error (I/O, missing or invalid manifest context)
- 2: parse error or usage/CLI argument error
- 3: resolution error
- 4: integrity error
- 5: render/check failed
- 6: verify failed (also used when `pray drift` finds drift)
- 7: network/fetch error
- 8: unsupported feature

Errors print to stderr. Successful primary output prints to stdout. Operator summary of the same codes lives beside the man page EXIT STATUS table.

---

## Implementation notes

Rust binary: `crates/pray-cli`. Ports: `rubygems/pray-cli`, `npmjs/pray-cli`. Each language implements the DSL. Shared parse contracts live in `testdata/shared`.

## Registrar

CLI verbs in the reference `Command` enum: manifest, init, prayer-init, repo-init, install, add, remove, update, unlock, render, plan, apply, verify, drift, format, package, publish, yank, token, search, login, serve, confess, list, outdated, explain, vendor, clean, tree, sync, trust, upgrade, version, completion.

## Unresolved questions

Whether RFC 0100 lists only conformance-level commands. Recommendation: this RFC lists the reference verbs; the man page lists the rest; RFC 0112 binds help, man, and exit codes.

Whether Ruby and TypeScript MUST implement every Rust verb or MAY lag with exit 8.
