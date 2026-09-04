# RFC 0070: Reference implementation

- Feature Name: reference-implementation
- Type: Informational
- Status: Stable
- Describes: 1.8.1
- Created: 2026-08-17
- Author: Andrei Makarov
- Relates: RFC 0040, RFC 0100, RFC 0101

## Summary

The workspace has four crates. This RFC records what the tree contains and what “reference implementation” means until RFC 0100 ships a fixture matrix.

## Motivation

Contributors looking for `pray-parser`, `pray-resolve`, `pray-lock`, `pray-render`, `pray-package`, `pray-distribution`, and `pray-verify` will not find those crate names. The modules live in `pray-core` (RFC 0101).

## Guide-level explanation

Workspace members: `pray-core` (parse, resolve, lock, render, verify, registry, trust), `pray-cli` (binary `pray` plus serve/publish/sync), `pray-transport` (HTTP/SSH/torrent/P2P/federation types), `pray-bench`.

Ruby and TypeScript `pray-cli` packages ship the same executable name and share destination parse fixtures. Each language implements the DSL.

Until RFC 0100 says otherwise, the Rust CLI is the reference when runtimes disagree.

## Reference-level explanation

This section is the product contract for this concern. Where it disagrees with Implementation notes, Implementation notes record what the reference CLI does today. A follow-on RFC records the gap.

The former snapshot suggested crate names that do not exist. RFC 0101 is the workspace map: pray-core, pray-cli, pray-transport, pray-bench.

### 9. Repository layout

#### Recommended project layout:

```
Prayfile
Prayfile.lock
tool-specific instruction files

.pray/cache/                # ignored by default
.pray/vendor/               # optional, committed only in hermetic/offline mode

.agents/                     # skills and other project agent inputs
```

Recommended `.gitignore`:

```
.pray/cache/
```

Depending on repository policy, rendered target files may be committed or ignored. Rendered files are usually committed because current inference tools commonly read repository-visible files, not `Prayfile` directly.

---

### 10. Commit policy

Default: commit Prayfile, Prayfile.lock, and rendered targets when tools need them; ignore cache and state.

Personal local: commit Prayfile; optionally lock; ignore generated tool output, cache, state.

Offline / archival: also commit `.pray/vendor` and generated files if targets need them.

---

### 54. CI workflow

Recommended CI:

```
pray install --frozen
pray verify --strict
pray drift
```

CI must fail when:

- lockfile is missing
- lockfile needs update
- package hash mismatch exists
- managed span checksum or line position mismatch exists
- removed prayer or orphan marker detected
- custom implementation detected inside a managed span
- target output exceeds hard limit
- package source unavailable and not vendored
- package is yanked and strict mode forbids it

---

### 55. Cache layout

Default cache location: `$PRAY_HOME/cache`

If `PRAY_HOME` is unset:

- Unix-like: `~/.cache/pray`
- Alternate desktop layout A: `~/Library/Caches/pray`
- Alternate desktop layout B: `%LOCALAPPDATA%\pray\cache`

Recommended cache structure:

```
cache/
  blobs/
    sha256/
      ab/
        abcdef...
  packages/
    sample/
      base/
        1.4.3/
```

Cache must be safely deletable.

Registry packages use a project-local cache beside other project state:

```
.pray/cache/registry/<namespace>/<name>/<version>/<source-hash>
```

Package identity MUST contain exactly two non-empty, path-safe segments. The
version MUST be one path-safe segment. `source-hash` is the first 16 lowercase
hexadecimal characters of SHA-256 over the exact source key used for
resolution. Artifact and tree hashes validate cache content; they are not cache
path inputs.

---

### 56. State file

`.pray/state.json` is local and ignored.

May contain: last render hashes, manual edit detection data, cache hints, local file hashes, tool discovery result

It must not be required for reproducible install. Deleting it must be safe.

---

### 57. Vendor mode

Vendor mode copies package contents into `.pray/vendor/`.

Used for: offline work, archival, regulated environments, private distribution without registry availability

Vendor mode must preserve package tree hashes. Vendor directory can be committed.

---

### 67. Semantic versioning for packages

Use semver. Patch: clarification without material behavior change. Minor: backward-compatible new guidance/export. Major: behavior-changing policy, removed/renamed export, stricter defaults. Do not hide meaningful changes in patch.

---

### 68. Changelog

Recommended package file: `CHANGELOG.md`

Package manifest may declare:

```manifest
spec.changelog_uri = "https://example.com/sample/webapp/CHANGELOG.md"
```

`pray update` should display changelog entries when available.

---

### 69. Package author best practices

Small topic-specific exports; skills for long procedures; templates for reusable forms; no vague/generic advice, private data, secrets, or conflicting instructions; document when to use each export and target compatibility; changelog + semver. Prefer `sample/base`, `sample/security`, … over `sample/everything`.

---

### 70. Repository author best practices

Short project-local instructions; reusable material in packages; commit lockfile; review generated diffs; frozen CI; intentional updates; do not edit generated files; split local context by topic; remove dead instructions; small roots; skills for longer material.

---

### 74. Minimal v1 implementation scope

Reference implementation should support:

- Prayfile parser
- prayspec parser
- Prayfile.lock reader/writer
- registry source
- git source
- path source
- tarball source
- package manifest validation
- semver resolver
- content hash verification
- tree hash verification
- tool_a target
- tool_b target
- managed rendering
- local append files
- pray markers
- install
- update
- render
- drift
- verify
- package
- static publish
- frozen CI mode
- offline cache mode

Do not implement in v1:

- AI-assisted rewriting
- autonomous learning
- session-end hooks
- package install scripts
- complex policy engine
- marketplace ranking
- telemetry
- automatic secret scanning beyond basic warnings
- dynamic host-language execution

---

### 75. Suggested reference implementation architecture

Modules: `pray-cli`, `pray-core`, `pray-parser`, `pray-resolve`, `pray-lock`, `pray-render`, `pray-package`, `pray-distribution`, `pray-verify`.

Layers: CLI → core orchestration → parser (Prayfile/prayspec DSL) → model (manifest, package, lock, render plan) → resolve (semver, sources, graph, lock writes) → render (fetch, hash/signature verify, adapters, markers, churn-minimal write) → doctor (drift, conflicts).

Deps: CLI args, codecs, lock/adapter formats, version constraints, hashing, UTF-8 paths, archives, compression, fs walk, text diff, structured errors.

Parser: strict hand-written or combinators. No host-language embed. No eval.

---

### 76. Open specification repository layout

```
prayfile-spec/
  README.md CHANGELOG.md LICENSE
  schema/   # manifest, lockfile, registry, package JSON schemas
  examples/ # minimal, webapp, multi-target, private-registry
  fixtures/ # parser, prayspec, resolver, render, verify
  rfcs/     # RFC 0001 process; template 0000; numbered design units
```

The specification repository in this tree uses `rfcs/` at the repository root (not a nested `prayfile-spec/` directory). RFC 0111 retired the snapshot; RFCs in this directory are the contract.

Implementation tree mirrors the module split in section 75 under `crates/`.

---

### 77. First milestone

Parse Prayfile and prayspec; local path packages; exact resolve; write lock; render `INSTRUCTIONS.md`; verify. No registry.

```
pray init --targets tool_a
pray add local/base --path ../agent-packages/base
pray install
pray verify
```

---

### 78. Second milestone

Package build, `.praypkg`, tree hash, tarball and static registry sources, `install --locked` / `--frozen`, `render --check`, diff.

---

### 79. Third milestone

Git source, update, semantic diff, additional targets/adapters, vendor and offline modes, static publish, conformance fixtures.

---

## Implementation notes

Quality program: rustfmt, clippy, cargo-deny, fuzz README, mutants.toml.

## Drawbacks

Calling three language CLIs “reference” without a shared fixture matrix will fork behavior. RFC 0100 is the mitigation.

## Rationale and alternatives

Split crates now to match the former crate diagram: compile-time and API-surface cost without a second Rust consumer. Keep the monolith until a stable library API is needed (RFC 0101).

Replace polyglot CLIs with wrappers around the Rust binary: simpler conformance, worse native packaging on RubyGems/npm. Current choice favors native ecosystem install.

## Unresolved questions

Whether `pray-core` should export a documented Rust API for embedders or remain a CLI implementation crate.

How much of `registry_*` should move into `pray-transport`.
