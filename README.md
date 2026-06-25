# pray & Prayfile

**A package manager for the language placed before inference.**

Prayfile is an open specification for reproducible inference input composition.

It lets projects declare shared instructions, policies, memories, templates, review checklists, formatting rules, and workflows in one place; resolve them deterministically; lock exact versions and hashes; preserve original source fragments; and render tool-specific outputs with compact provenance markers.

The goal is simple: treat pre-inference input as a dependency.

**Status:** Draft specification v0.1 — spec-first experiment.
The open specification is the primary focus. The `pray` reference CLI is described in `SPEC.md`.

## Why

Modern inference engines increasingly rely on surrounding input files such as `AGENTS.md`, `CLAUDE.md`, instruction libraries, prompt templates, review checklists, memories, formatting rules, and workflow notes.

These files shape input conditions and output behaviour, but they are often distributed manually through copy-paste.

As shared input libraries grow, repositories accumulate stale instructions, hidden drift, inconsistent updates, and difficult rollbacks. The material placed before inference becomes harder to audit, review, restore, and maintain.

Prayfile provides a reproducible way to package, version, distribute, compose, render, and preserve inference input material.

Instead of manually copying files between repositories, teams declare input dependencies, resolve them deterministically, lock exact versions and content hashes, and render reproducible outputs for supported tools.

The lockfile records the resolved state, including checksums of original source fragments. A local compressed cache may preserve the exact original pieces fetched from their sources. Together, the lockfile and cache make input consistency, verification, rollback, and backup possible without relying on mutable upstream state.

Rendered files contain only the material intended to affect inference, plus compact citation markers that point back to `Prayfile.lock`.

## What problem does Prayfile solve?

Prayfile solves reproducible composition and synchronization of shared inference input material across repositories, teams, and tools.

Its main concern is input drift: the gradual divergence of instructions, policies, templates, memories, formatting rules, and workflow assumptions between projects.

Prayfile also addresses preservation. Resolved input material should remain verifiable and recoverable even if an upstream source changes, disappears, or becomes temporarily unavailable.

## Is this a prompt framework?

No.

Prompts, templates, memories, workflows, policies, review rules, formatting rules, and instruction sets may all be packaged using Prayfile, but the durable problem is not prompt design itself.

The durable problem is packaging and distributing the material that gets placed before inference.

Prayfile focuses on dependency management, version locking, deterministic resolution, reproducible rendering, provenance, verification, preservation, planning, publishing, signed feedback, and drift detection.

It does not define an inference runtime. It does not require a specific model provider. It does not prescribe one correct format for intelligence, collaboration, review, or automation.

It manages pre-inference material as data.

## What does the tool do?

Prayfile:

* resolves declared input dependencies
* locks exact versions and content hashes
* records verifiable source checksums in `Prayfile.lock`
* may keep a local compressed cache of original source fragments
* renders deterministic inference-facing files
* tracks provenance through compact render markers
* supports consistency checks, rollback, and backup of resolved input material
* supports plan/apply-style review before modifying rendered files
* detects drift in generated blocks
* publishes signed packages to distribution points
* sends signed usage feedback through `pray confess`
* can serve a local or self-hosted distribution point through `pray serve`
* enables reviewable updates through normal version control workflows
* avoids arbitrary package code execution

## Operating model

Prayfile combines two familiar models.

From Bundler, it takes dependency declaration, resolution, version constraints, lockfiles, checksums, package sources, updates, and deterministic installation.

From Terraform, it takes planning, materialization, drift detection, and reviewable changes.

Prayfile resolves input packages like Bundler and materializes inference-facing files with Terraform-style planning, verification, and drift detection.

```text
Prayfile        declares desired input dependencies
Prayfile.lock   records exact resolved state
cache           preserves original source fragments
renderer        formats target-specific files
target files    affect inference when loaded by tools
```

The lockfile is for truth.
The cache is for recovery.
The rendered file is for influence.

## Repository model

Pray packages do not need to live expanded in the repository.

The usual repository layout is:

```text
Prayfile
Prayfile.lock
AGENTS.md
CLAUDE.md
.github/copilot-instructions.md

.pray/cache/     # ignored by default
.pray/vendor/    # optional, committed only for hermetic/offline mode
```

`Prayfile` is committed because it declares desired input dependencies.

`Prayfile.lock` is committed because it records the exact resolved state.

Rendered target files are usually committed because current inference tools commonly read repository-visible files, not `Prayfile` directly.

The local cache is ignored by default. It may be committed or archived only in hermetic, regulated, air-gapped, or long-term preservation modes.

Expanded packages are not committed by default. They are dependencies, like gems.

## Materialization and effect

Prayfile distinguishes between references and loaded input.

A package reference, bookmark, source path, import marker, or lockfile entry is not equivalent to full rendered text.

Only material that is rendered, expanded, or otherwise loaded into the inference input should be assumed to affect inference directly.

A bookmark may still affect discovery, routing, retrieval, or attention if the consuming tool reads it. But a bookmark is not the same as the material it points to.

Therefore:

```text
Prayfile        does not affect inference unless read by a tool
Prayfile.lock   does not affect inference unless read by a tool
cache           does not affect inference unless read by a tool
rendered files  affect inference when loaded by the target tool
```

Prayfile’s default assumption is conservative: rendered target files are the primary inference-facing artifacts.

## Prayer and silence

Prayfile manages both prayer and silence.

Prayer is the selected language rendered before inference.

Silence is the deliberate exclusion of resolved material from rendered input.

This matters because inference is shaped by both presence and absence. Extra words affect output. Repeated rules affect output. Formatting affects output. But silence also affects output: not rendering a fragment, not including verbose provenance, not loading irrelevant package parts, not leaking source metadata, and not letting every dependency speak into every target.

Silence must be explicit and reproducible. If a package contains fragments that are resolved but excluded from a target, that exclusion should be recorded in `Prayfile.lock`.

```text
Prayfile        declares what may speak and what must stay silent
Prayfile.lock   records selected, rendered, and silenced fragments
cache           preserves original fragments
target files    contain only what is meant to affect inference
```

The rendered file should contain influence, not bookkeeping.

## Philosophy

Inference input is not passive documentation.

It is the language placed before inference: the surrounding material that affects what a model notices, ignores, repeats, refuses, prioritizes, imitates, formats, or treats as important.

This includes explicit instructions, examples, checklists, project policies, workflow notes, style guides, tool descriptions, memories, conventions, file names, headings, ordering, repetition, and local vocabulary.

Some of this input is declarative. Some of it is procedural. Some of it is stylistic. Some of it is inherited accidentally from previous work. In all cases, it becomes part of the conditions under which inference happens.

That makes input operational.

If input can shape output, then input should be inspectable. If it can drift, then it should be locked. If it can be shared, then it should have provenance. If it can disappear upstream, then it should be recoverable. If formatting can change behaviour, then rendered output should be reproducible.

Prayfile exists because the language around an inference engine is already infrastructure.

Most teams treat this infrastructure as loose text. They copy files between repositories, paste fragments into new tools, modify local instructions without traceability, and slowly lose the relationship between source material and rendered output.

Prayfile treats that material as a dependency graph.

Not because inference becomes fully deterministic. It does not. Output remains probabilistic, provider-specific, model-specific, tool-specific, and situation-sensitive.

The point is narrower and more practical: the pre-inference input should not be the least reproducible part of the system.

## Why the name Prayfile?

The name is intentional.

`Agentfile` is too narrow. It suggests the file belongs to an agent runtime, while the specification is about input packages that may be consumed by many tools: coding assistants, chat systems, review bots, documentation generators, local inference wrappers, automation tools, and future interfaces that may not call themselves agents.

`Cookfile` suggests recipes and execution. That is close to dependency composition, but misleading for this project. Prayfile does not cook, run, orchestrate, or execute package code. It resolves and renders input data.

`Mantrafile` is also close. Inference input often works through repetition, phrasing, remembered instruction patterns, and stable verbal forms. But mantra points too strongly toward repeated prompt text and too weakly toward dependency management, version locking, provenance, recovery, formatting, and multi-source composition.

`Prayfile` describes a broader mechanism.

In religious practice, prayer is not only information transfer. A prayer is language used as orientation. It may ask, praise, confess, remember, surrender, repeat, listen, or prepare the believer for a different relation to experience.

For true believers, prayer is often effective not merely because of the words as semantic content, but because of the whole form: repetition, breath, posture, rhythm, silence, community, memory, expectation, and attention. The same words repeated in a structured setting can alter perception, emotion, judgment, and state of mind.

Prayfile uses this analogy carefully.

Inference engines are not gods. Prompting is not worship. The specification does not make mystical claims.

The useful point is structural: repeated, formatted language placed before action can prepare a system toward certain behaviour.

Modern inference engines are affected by prior input. Instructions, examples, policies, memories, formatting, headings, order, repetition, and local conventions all influence output. Before the model produces output, the surrounding input asks it to behave in a certain way. That request may be explicit, implicit, procedural, stylistic, or structural, but it is still part of the computation.

In that limited sense, pre-inference material behaves like a technical prayer: language placed before uncertain response, intended to shape attention, judgment, refusal, style, formatting, and action.

Prayfile does not make this mystical. It makes it auditable.

If input has power, then it should have checksums. If instructions affect output, then they should be versioned. If formatting changes behaviour, then rendered output should be reproducible. If teams share input material, then provenance, rollback, and backup should exist.

The name keeps the philosophical point visible without changing the technical scope.

Prayfile is a package manager for the language placed before inference.

## Does Prayfile execute package code?

No.

Prayfile packages are data. Resolution and rendering are deterministic and do not require executing arbitrary package code.

This keeps package use auditable and reduces the security risks of distributing executable tooling alongside inference input material.

## Distribution points

Prayfile packages may be fetched from distribution points.

A distribution point is a registry-like source for package metadata, package archives, signatures, checksums, usage feedback, and optional web documentation.

An example public or private distribution point could be:

```text
https://prayers.kisko.dev
```

A distribution point should expose a minimal API for:

* package lookup
* version listing
* package archive download
* checksum retrieval
* signature retrieval
* publisher identity lookup
* package publishing
* package yanking or deprecation metadata
* signed usage feedback submission
* package acceptance and rejection statistics
* optional human-readable package pages

The distribution point is not part of inference. It is part of package discovery, publishing, verification, feedback, and preservation.

Prayfile should also support direct sources such as local paths, git repositories, archive URLs, and vendored `.praypkg` files. A centralized distribution point is useful, but not required.

## Publishing and signatures

Publishing changes the shared material placed before inference, so publishing must be treated as a high-trust operation.

A hardened distribution point should require strong authentication for publishing.

Recommended publishing requirements:

* account authentication
* two-factor authentication
* passkey support
* explicit passkey check on publishing
* package signature on publish
* SSH signing key support
* publisher identity recorded in package metadata
* immutable package archives after publish
* yanking or deprecation instead of silent replacement
* append-only audit log for publish events
* checksum verification before and after upload

Package archives should be signed.

Signatures may be produced with supported signing methods such as passkeys, SSH signing keys, or other configured project keys.

`pray publish` should create the package archive, compute content hashes, sign the package, and upload the package plus metadata to the selected distribution point.

`pray install`, `pray update`, and `pray verify` should verify package hashes and signatures according to policy.

`Prayfile.lock` should record enough information to verify that the resolved package still matches the expected archive, source fragments, publisher identity, and signature policy.

The package archive is the object being distributed. The lockfile is the local record of what was accepted.

## Confession feedback

Prayfile may support signed usage feedback through `pray confess`.

A confession is feedback from a user, project, team, or automation about a specific prayer package, version, fragment, rendered span, or resolved lockfile entry.

The basic confession result is binary:

```text
accepted
rejected
```

A confession may include an optional free-form note.

Examples:

```sh
pray confess kiskolabs/rails-review --version 0.4.2 --accepted
pray confess kiskolabs/rails-review --version 0.4.2 --rejected --note "Too broad for small maintenance tasks."
pray confess --from-lock p7f3k9m2 --accepted --note "Useful authorization checklist."
```

A confession is not an inference result, benchmark, review score, or package rating by default. It is a signed signal that a specific resolved prayer was accepted or rejected in a specific usage context.

Distribution points may aggregate confessions to show package usefulness, rejection patterns, compatibility issues, version regressions, or fragment-level feedback.

Confession submission should be hardened.

Recommended confession requirements:

* authenticated account or configured project identity
* explicit passkey verification before submission
* signed confession payload
* SSH signing key support
* optional passkey-backed signature support
* package/version/span reference
* accepted or rejected status
* optional free-form note
* timestamp
* distribution point identity
* lockfile reference when available
* network fingerprint
* append-only audit log entry

The signed confession payload should include the package reference, version, status, note hash, timestamp, signer identity, signing key identity, distribution point identity, and network fingerprint.

The optional free-form note may be stored as clear text or as a separately hashed/encrypted field depending on distribution point policy.

The network fingerprint is an additional verification signal, not an identity source and not a replacement for signatures. It may include a privacy-preserving hash of server-observed network metadata, client-declared environment metadata, or both, according to the distribution point policy.

Distribution points should avoid storing raw network metadata unless explicitly configured to do so.

`pray confess` should show the payload before signing.

A confession should be rejected by the distribution point when:

* the signature is invalid
* the passkey check is missing when required
* the SSH signing key is not trusted for the account or project
* the package/version/span reference cannot be resolved
* the network fingerprint policy is not satisfied
* the payload has been replayed
* the distribution point policy rejects the note or metadata

Confessions should be append-only. Corrections should be submitted as new confessions, not by silently editing old ones.

## `pray serve`

The reference CLI may include a `serve` command.

`pray serve` can expose a local or self-hosted distribution point using the same package format, metadata format, verification rules, and feedback rules as public distribution points.

This keeps the reference tool useful on both sides:

```text
consumer side     resolve, install, render, verify, confess
publisher side    package, sign, publish, serve
```

Possible uses:

* local package testing
* private team registry
* air-gapped distribution
* CI fixture server
* offline documentation browser
* small public registry
* package archive mirror
* private feedback collection
* confession review and moderation

Example:

```sh
pray serve --root ./prayers --host 127.0.0.1 --port 7429
```

The server should provide API endpoints for package metadata, archive retrieval, signature retrieval, and confession submission. It may also provide simple human-readable HTML pages.

The server is a distribution and feedback mechanism, not an inference runtime.

## Bundled assets

The Rust reference implementation should not bundle binary assets.

No fonts, images, icons, databases, compressed binary blobs, or opaque static bundles should be embedded into the executable.

Plain text assets are acceptable when needed for the built-in server or help output.

Allowed embedded assets:

* HTML templates
* CSS
* plain text help
* plain text schema examples
* small text fixtures required for tests or diagnostics

Avoiding bundled binary assets keeps the executable inspectable, reproducible, portable, and easier to package across platforms.

The built-in web interface, if provided, should be minimal and text-first.

HTML and CSS are enough.

## How does Prayfile keep input consistent?

`Prayfile.lock` records the exact resolved dependency graph, selected versions, source references, content hashes, silenced fragments, render targets, renderer versions, formatting options, signature metadata, distribution point metadata, confession references, provenance metadata, and **managed span records**.

Each managed span (a **prayer** rendered between pray markers) has a lockfile entry that stores:

* marker ID
* target file path
* **ideal checksum** — semantic hash of the expected managed body (content between markers, excluding pray comment lines)
* **line positions** — opening and closing marker line numbers in the target file
* provenance — package, export, source fragment checksum, silenced flag, and related metadata

The lockfile is the authoritative record of what should appear in each managed span and where it should live.

This allows projects to verify that rendered files still match the resolved state.

A local compressed cache may store the original source fragments used during resolution. This gives the project a recoverable copy of the resolved input material, independent of mutable upstream repositories, registries, or URLs.

In short:

* the lockfile verifies what was resolved and where each prayer should appear
* ideal checksums detect custom edits to managed content
* line positions detect marker movement and missing spans
* the cache preserves what was resolved
* signatures verify who published or approved the package
* confessions record signed acceptance or rejection feedback
* rendered outputs contain what was selected to speak
* silenced fragments remain out of inference-facing files

## What is preserved?

Prayfile is designed to preserve the relationship between source fragments and rendered outputs.

A rendered file may contain merged instructions from several packages, local overrides, target-specific formatting, generated boundaries, and compact citation markers. Without a lockfile and cache, it can become difficult to reconstruct why the output looks the way it does.

Prayfile should make the following recoverable:

* which packages were selected
* which source fragments were used
* which source fragments were silenced
* which versions or revisions were resolved
* which content hashes were expected
* which package archive was accepted
* which signatures were verified
* which publisher identity was recorded
* which distribution point was used
* which confessions were submitted or referenced
* which local overrides were applied
* which renderer produced the final output
* which target file received each fragment
* which formatting rules affected the rendered output

The rendered output is not just a blob. It is a traceable composition.

## Render markers

Rendered target files should not duplicate the dependency graph, package list, source URLs, source hashes, or provenance records already stored in `Prayfile.lock`.

Rendered files use compact citation markers, not provenance blocks.

A marker is an opaque reference into `Prayfile.lock`. It identifies a managed rendered span but does not explain it.

Markdown targets use this canonical marker form:

```md
<!-- pray:p7f3k9m2 -->

...rendered content...

<!-- pray:p7f3k9m2 -->
```

The same marker appears exactly twice for one managed block.

The first occurrence opens the block.

The second occurrence closes the block.

Nested pray blocks are invalid.

Unmatched markers are invalid.

A marker ID must be opaque. It must not encode package names, topic names, versions, hashes, source paths, or semantic labels.

Marker IDs must use only lowercase ASCII letters and digits.

Marker IDs should be 8–16 characters.

Marker comments must appear on their own lines.

The purpose of a marker is region identity, drift detection, and lockfile lookup.

Rendered output cites.
Lockfile explains.
Cache preserves.

## Ignore marker

Rendered files may include a compact header marker near the beginning of the file:

```md
<!-- pray:0 ignore-comments -->
```

This marker declares that `pray` comments are render markers and should not be interpreted as instruction content.

The marker is advisory for inference behaviour and binding for Prayfile tooling.

Prayfile tooling must ignore pray comments when computing semantic content hashes.

Prayfile tooling may also compute exact file hashes that include marker bytes.

Therefore, implementations may track both:

```text
semantic hash  = rendered content without pray markers
file hash      = exact target file bytes including pray markers
```

The semantic hash answers: did the meaningful rendered input change?

The file hash answers: did the physical target file change?

## Formatting rendered files

`pray format` normalizes render markers in target files.

It should:

* place every pray marker on its own line
* normalize marker spacing to the canonical form
* ensure a blank line after an opening marker when the target format allows it
* ensure a blank line before a closing marker when the target format allows it
* reject nested managed blocks
* reject unmatched markers
* reject malformed marker IDs
* reject duplicate marker IDs in the same target unless explicitly allowed by the lockfile
* preserve non-managed content outside pray blocks

Canonical Markdown form:

```md
<!-- pray:p7f3k9m2 -->

...rendered content...

<!-- pray:p7f3k9m2 -->
```

Invalid forms include inline markers, markers with special characters, markers with whitespace inside the ID, unmatched markers, and manually nested managed blocks.

## Planning, verifying, applying, and drift

Prayfile separates **detection** from **materialization**.

### `pray plan`

Computes the changes that would happen to resolved packages, lockfile entries, cache contents, and rendered target files.

### `pray apply`

Materializes the planned changes.

After render, `pray apply` **refreshes** `Prayfile.lock`:

* updates ideal checksums for each managed span
* updates opening and closing marker line positions
* adds, updates, or removes managed span records when prayers are introduced, re-rendered, or silenced

`apply` is the only normal command that should rewrite managed span checksums and line positions after intentional materialization.

### `pray verify`

Read-only consistency check against the current lockfile and on-disk target files.

For every managed span record, `pray verify` locates the marker pair in the target file and checks:

* the span exists (both opening and closing markers are present)
* the managed body checksum matches the lockfile **ideal checksum**
* the marker **line positions** match the lockfile positions

`pray verify` reports mismatches. It does **not** modify `Prayfile.lock` or target files.

Position-only changes are still reported: if the ideal checksum still matches but marker lines moved, verify reports **position drift**.

Content changes are reported when the ideal checksum does not match: the on-disk prayer body differs from the locked ideal even though markers may still be present. This catches custom implementation edits inside a managed span.

Missing spans are reported when a lockfile record exists but the marker pair is absent — the prayer was removed from the target file.

### `pray drift`

Drift detection is a superset of verify.

`pray drift` reports everything `pray verify` reports, plus drift against the **current renderer output**:

* **custom implementation** — markers remain, but the managed body no longer matches the locked ideal checksum (hand-edited replacement text)
* **removed prayer** — lockfile still records a managed span, but the marker pair is gone from the target file
* **position drift** — content checksum still matches the ideal, but marker line positions changed
* **renderer drift** — on-disk file matches the lock, but a fresh render from current packages and `Prayfile` would produce different ideal checksums or different prayers
* **orphan marker** — a pray marker pair exists in a target file with no matching lockfile managed span record

`pray drift` is for review and CI. It does not refresh the lockfile.

To accept intentional changes after plan review, run `pray apply`.

### `pray render`

May be used as a non-interactive rendering command for CI and automation. Rendering alone does not replace the plan/apply lock refresh contract unless the invocation is explicitly documented as also updating managed span records (for example `pray apply` or `pray install` after resolve).

### Other commands

`pray publish` packages, signs, and uploads a package to a distribution point.

`pray confess` signs and submits acceptance or rejection feedback to a distribution point.

`pray serve` runs a local or self-hosted distribution point.

Example command set:

```sh
pray init
pray add kiskolabs/rails-review
pray install
pray update
pray plan
pray apply
pray render
pray format
pray verify
pray drift
pray package
pray publish
pray confess
pray serve
pray vendor
pray clean
```

## Package storage modes

Prayfile supports several storage modes.

Default mode:

```text
Prayfile
Prayfile.lock
AGENTS.md
.gitignore: .pray/cache
```

Use this for most repositories. Packages are fetched from sources and cached locally. Rendered files are committed for current tool compatibility.

Hermetic mode:

```text
Prayfile
Prayfile.lock
AGENTS.md
.pray/vendor/*.praypkg
```

Use this for air-gapped repositories, regulated work, long-term archival, or cases where upstream availability cannot be trusted.

Generated-only mode:

```text
Prayfile
Prayfile.lock
```

Use this only when every developer, CI worker, and inference tool reliably runs `pray render` before use. This mode is cleaner, but less safe with current inference tooling.

Served mode:

```text
Prayfile
Prayfile.lock
AGENTS.md

internal distribution point:
  pray serve
```

Use this when a team wants a private package source without operating a separate registry implementation.

Feedback mode:

```text
Prayfile
Prayfile.lock
AGENTS.md

feedback:
  pray confess --accepted
  pray confess --rejected --note "..."
```

Use this when package consumers want to send signed acceptance or rejection feedback to a distribution point.

## What is not a goal?

Prayfile is not a prompt engineering doctrine.

It does not decide what a good instruction is. It does not claim that shared input material is always beneficial. It does not replace human review. It does not remove responsibility from maintainers. It does not guarantee better model behaviour.

It also does not attempt to turn input material into executable software.

The specification is about packaging, resolving, locking, caching, rendering, formatting, planning, applying, verifying, publishing, signing, serving, confessing, silencing, and tracing files and fragments that shape inference input and output formatting.

Quality remains a human and project-level concern.

## Ruby and Rails integration (planned)

Prayfile is language-independent, but Ruby and Rails are a natural first host for runtime prayer support.

A planned Ruby gem would let web applications that incorporate inference load and compose prayers at runtime — without reimplementing resolve, lock, or render inside the app process.

### Split of responsibility

| Layer | Owner | Role |
|-------|-------|------|
| `pray` CLI | reference implementation | resolve, lock, render, verify, drift, publish, serve |
| Ruby gem | planned host integration | read lock/cache, assemble inference input, optional Rails hooks |
| Rails app | application | declare `Prayfile`, commit lock + rendered targets, call inference with composed input |

The gem would not replace `pray`. Development and CI still run `pray plan`, `pray apply`, `pray verify`, and `pray drift`. The gem consumes the artifacts they produce.

### What the gem could provide

* load managed spans from `Prayfile.lock` (ideal checksums, provenance, silenced fragments)
* resolve prayer bodies from `.pray/cache`, vendored `.praypkg`, or distribution points
* assemble inference-facing text for a named target (for example `AGENTS.md`, a custom `:rails` target, or an in-memory buffer)
* expose the same drift semantics as the CLI: custom implementation, removed prayers, position drift, orphan markers
* Rails integration: initializers, request/job-scoped context builders, generators, and rake tasks that delegate to `pray` where shelling out is appropriate

### Typical Rails workflow

```text
Prayfile + Prayfile.lock   committed in the app repo
AGENTS.md / CLAUDE.md      committed rendered targets (today's tool compatibility)
.pray/cache/               ignored locally; populated by pray install

CI:  pray verify --strict && pray drift
App: Prayer::Context.for(:inference).to_s  # example API; not normative yet
```

### Design constraints

* prayers remain data — no arbitrary package code execution in the gem either
* lockfile managed span records remain authoritative for verify/drift
* the gem should not fork marker or checksum rules; it implements the same contracts as `README.md` and `SPEC.md`
* gem name and API are not finalized; `prayer`, `prayfile`, and `prayers` are candidates

## Is the specification final?

No.

The project is experimental. Terminology, formats, package structure, resolver rules, rendering targets, cache behaviour, registry design, distribution APIs, marker syntax, formatting rules, signing policy, confession policy, and implementation details may evolve as the model is validated through real-world use.

The specification is currently the main area of development.

## Core model

| Concept                | Role                                                                                   |
| ---------------------- | -------------------------------------------------------------------------------------- |
| `Prayfile`             | Human-authored input dependency manifest                                               |
| `Prayfile.lock`        | Machine-authored resolved state, including ideal checksums and marker line positions per managed span |
| managed span           | Lockfile record for one prayer between pray markers in a target file                                 |
| ideal checksum         | Semantic hash of expected managed span body stored in `Prayfile.lock`                                |
| `*.prayspec`           | Package definition                                                                     |
| `*.praypkg`            | Package archive                                                                        |
| distribution point     | Registry-like source for packages, metadata, checksums, signatures, feedback, and docs |
| package signature      | Verifiable publisher approval for a package archive                                    |
| confession             | Signed acceptance or rejection feedback for a resolved prayer                          |
| network fingerprint    | Additional verification signal attached to a signed confession                         |
| local cache            | Compressed storage for original resolved source fragments                              |
| local vendor directory | Optional committed package cache                                                       |
| rendered target file   | Inference-facing materialized output                                                   |
| pray marker            | Compact citation into the lockfile                                                     |
| silenced fragment      | Resolved fragment intentionally excluded from target output                            |
| `pray`                 | Reference CLI                                                                          |
| `pray serve`           | Built-in distribution point server                                                     |

## Design principles

```text
Declare input.
Resolve deterministically.
Lock exactly.
Verify by checksum.
Sign packages.
Harden publishing.
Collect signed feedback.
Cache original fragments.
Render reproducibly.
Cite compactly.
Format safely.
Plan before applying.
Detect drift.
Serve without extra machinery.
Never execute package code.
Never hide updates.
Keep diffs small.
Preserve provenance.
Support rollback.
Respect silence.
Avoid bundled binary assets.
```

## Repository layout

| Path             | Purpose                                 |
| ---------------- | --------------------------------------- |
| `SPEC.md`        | Normative specification                 |
| `AGENTS.md`      | Contributor and inference tool workflow |
| `spec/README.md` | Test coverage guidelines                |

## Read the specification

Start with `SPEC.md` for:

* file formats
* resolver behaviour
* lockfile semantics
* checksum verification
* local cache behaviour
* package structure
* distribution point API
* publishing and signing policy
* confession feedback policy
* built-in serving behaviour
* registry design
* rendering targets
* render marker rules
* formatting behaviour
* managed span records (ideal checksums and line positions)
* plan/apply behaviour
* verify and drift detection
* CLI commands

## Contributing

Bug reports, design discussions, examples, and pull requests are welcome.

Please read `CONTRIBUTING.md` before submitting changes.

The specification is currently the primary area of development and feedback.

## Security

Please do not disclose security vulnerabilities through public issues.

See `SECURITY.md` for responsible disclosure instructions.

Publishing should require strong authentication, explicit publish-time verification, and signed package archives.

Confession submission should require explicit verification, signed payloads, replay protection, and privacy-aware handling of network fingerprints.

## License

MIT. See `LICENSE.md`.
