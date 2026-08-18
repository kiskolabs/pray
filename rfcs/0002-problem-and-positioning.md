# RFC 0002: Problem and positioning

- Feature Name: problem-and-positioning
- Type: Informational
- Status: Stable
- Describes: 1.8.1
- Created: 2026-08-17
- Author: Andrei Makarov
- Relates: RFC 0010

## Summary

Teams copy instruction files between repositories. The copies drift, hide ownership, and resist rollback. Inference engines read those files as input. This RFC records the problem Prayfile already addresses: declare, lock, and render static text trees as dependencies.

## Motivation

Copied checklists and policy files fork silently. A library of review guidance declared in many application repositories needs one update path with a reviewable lock and render diff. A CI job that runs `pray install --frozen` and `pray verify --strict` should fail when a managed span was edited by hand.

## Guide-level explanation

Declare packages in Prayfile. Lock hashes and managed span records in Prayfile.lock. Rendered files hold opaque `pray:` markers that cite the lock (RFC 0030).

Packages are static text trees. There is no agent runtime, no hidden self-updater, and no install scripts. Resolve and render stay in the CLI. A later host-language adapter MAY read Prayfile.lock (reserved RFC 0106).

## Reference-level explanation

This section is the product contract for this concern. Where it disagrees with Implementation notes, Implementation notes record what the reference CLI does today. A follow-on RFC records the gap.

### 1. Summary

Prayfile is an open specification for reproducible pre-inference input composition: declare shared instructions and related input, resolve deterministically, lock versions and hashes, preserve source fragments, and render tool-specific outputs with compact pray markers that cite `Prayfile.lock`.

Core model:

- Prayfile: human-authored input dependency manifest
- Prayfile.lock: machine-authored resolved state
- *.prayspec: package definition
- *.praypkg: package archive
- distribution point: packages, metadata, checksums, signatures, feedback, docs
- pray: reference CLI

Static declarations only. No host-language execution. Parseable by any implementation.

Not a prompt framework or agent runtime. The durable problem is packaging and distributing material placed before inference. RFCs and the reference CLI evolve together; formats may still change.

---

### 2. Core positioning

Design principles:

```text
Declare input. Resolve deterministically. Lock exactly.
Verify by checksum. Sign packages. Harden publishing.
Collect signed feedback. Cache original fragments.
Render reproducibly. Cite compactly. Format safely.
Plan before applying. Detect drift.
Serve without extra machinery.
Never execute package code. Never hide updates.
Keep diffs small. Preserve provenance. Support rollback.
Respect silence. Avoid bundled binary assets.
```

Core values (normative indicators):

- Auditable traces: compact pray markers on managed spans; lockfile holds resolved state, ideal checksums, marker lines, source checksums, silence, provenance
- Temporal clarity: lock and drift show what changed; markers enable blame and rollback
- Measurable effects: manifest → lock → rendered bytes → diff; inference quality stays human-validated
- Security: static packages, hash-verified, path-safe, explicitly updated, optionally signed

Inference-input packaging will keep changing. Prayfile stays useful by pinning lock state, markers, diffs, integrity checks, and signed feedback as contracts.

---

### 3. Problem

Teams copy instruction files, templates, checklists, memories, formatting rules, and workflow notes between repositories. That yields duplication, stale context, noisy diffs, unclear ownership, hidden drift, hard rollback/audit, tool-specific conflicts, private-input leakage, and giant merged files.

Inference input shapes model behavior. Treat it as a dependency.

---

### 4. Goals

Priorities: human-readable files; small diffs; minimal generated output; deterministic install and render; explicit updates; lockfile reproducibility; cross-platform; any-language implementations; public/private/local/P2P distribution; no code execution; recovery, vendoring, CI validation; clear ownership; tool-neutral packages with tool-specific adapters; auditable provenance; supply-chain security for context packages.

```
less formatting religion
more stable meaning
less generated sludge
more readable reviewable context
```

---

### 5. Non-goals

Not: agent runtime, chat memory, session-end learning, prompt-injection firewall, secret manager, governance platform, marketplace ranking, background self-updater, hidden instruction mutator, autonomous context writer, human-review replacement, executable-code package manager, or a bet on any single agent workflow shape.

Self-recovery: reconstruct from Prayfile.lock. Self-update: explicit `pray update`. Never hidden mutation.

---

### 6. Naming

- manifest / lockfile: Prayfile, Prayfile.lock
- package: *.prayspec, *.praypkg
- distribution point: registry-like source
- CLI / project / crate: pray

The names Prayfile, Prayfile.lock, prayspec, and pray are fixed. Distribution repo root inside a larger checkout: `prayers/` (`pray repo init`). Resolve and render may be internal phases, not CLI aliases.

---

### 7. Ecosystem analogy

Closest reference: dependency manifest/lockfile ecosystems.

- Prayfile / Prayfile.lock ↔ manifest / lockfile
- *.prayspec / *.praypkg ↔ package spec / archive
- resolve / render ↔ lock step / materialize merged context (no direct code-dep equivalent)
- `pray verify` / `pray drift` ↔ integrity checks / lock+render diff

Difference: legacy registries may execute host code; Prayfile parses declarations only. Same supply-chain baseline (checksums, optional signing, vendoring) plus compact markers in rendered files.

Prayfile does not replace language package managers. A planned host-language adapter may load from `Prayfile.lock` and cache; it does not replace CLI resolve/render.

---

### 8. Terminology

- Prayfile: human-authored dependency manifest
- Prayfile.lock: machine-authored resolved state
- prayspec: package definition file
- agent package: versioned bundle of agent-context content
- export: named unit from a package (e.g. `webapp-review`, `testing-guidance`)
- target: agent tool or output environment (`tool_a` … `generic`)
- adapter: maps exports to target-specific files
- render: create target files from locked state
- managed file: generated file owned by pray
- local file: human-owned project file embedded into rendered output
- source: registry, static index, git, path, tarball, OCI, file share, …
- frozen install: refuses lockfile or generated-file updates
- annotation / claim: untrusted derived metadata, confession, score, or analysis
- render digest: hash of final injected bytes after render and normalization

---

### 81. Final principle

Do not make inference smarter with magic. Make inference input boring enough to trust.

```
declared input
locked input
verified input
rendered input with compact citations
recoverable source fragments
explicit silence
```

## Implementation notes

Workspace 1.8.1. Closest shipped loop: `pray add`, `pray install`, `pray plan`, `pray apply`, `pray verify`.

## Drawbacks

This file goes stale if public copy drifts from shipped commands. Keep it factual.

## Prior art

Bundler declare/lock. Terraform plan/apply/drift. Cargo.lock and npm lockfiles. Copy-based inference-input sync without hashes or markers.

## Unresolved questions

Whether “agent package” remains the right noun now that destination DSL uses `pray` declarations (RFC 0102).
