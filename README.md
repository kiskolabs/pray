# pray & Prayfile

A package manager for the language placed before inference.

Prayfile is an open specification for reproducible inference input composition.

It lets projects declare shared instructions, policies, memories, templates, review checklists, formatting rules, and workflows in one place; resolve them deterministically; lock exact versions and hashes; preserve original source fragments; and render tool-specific outputs with compact provenance markers.

The goal is to treat pre-inference input as a dependency.

The specification and the `pray` reference CLI evolve together. Numbered RFCs under `rfcs/` define the current contracts (RFC 0001, RFC 0111).

Website: [pray.kisko.dev](https://pray.kisko.dev)

## Install `pray`

The canonical GitHub repository is `kiskolabs/pray`.

Install the Rust `pray` CLI as a system-wide binary with Cargo:

```sh
cargo install pray-cli --locked
```

The [crates.io](https://crates.io/crates/pray-cli) package is `pray-cli` (the name `pray` is already taken on crates.io). Cargo installs the `pray` executable into your user bin directory, typically `~/.cargo/bin`.

From git:

```sh
cargo install --git https://github.com/kiskolabs/pray --locked pray-cli
```

If `~/.cargo/bin` is not already on your `PATH`, add this line to `~/.zshrc`, `~/.bashrc`, `~/.zprofile`, or `~/.bash_profile`:

```sh
export PATH="$PATH:~/.cargo/bin"
```

Shell completions (bash, zsh, or fish):

```sh
pray completion bash > ~/.local/share/bash-completion/completions/pray
pray completion zsh > ~/.zsh/completions/_pray
pray completion fish > ~/.config/fish/completions/pray.fish
```

Man page (from a checkout): `man ./docs/man/pray.1`. Exit codes: [docs/cli-exit-codes.md](docs/cli-exit-codes.md).

## Why

Modern inference engines increasingly rely on surrounding input files such as `AGENTS.md`, `CLAUDE.md`, instruction libraries, prompt templates, review checklists, memories, formatting rules, and workflow notes.

These files shape input conditions and output behaviour, but they are often distributed manually through copy-paste.

As shared input libraries grow, repositories accumulate stale instructions, hidden drift, inconsistent updates, and difficult rollbacks. The material placed before inference becomes harder to audit, review, restore, and maintain.

Prayfile treats that shared material as a dependency graph. Projects can inspect the selected input, review updates, and recover resolved material from a local cache when upstream sources change or disappear.

## Why use this with Git, Mercurial, Subversion, or CVS?

CVS, Git, Mercurial, and Subversion record the history of one working tree: commits, branches, diffs, blame, and rollback of that tree.

Shared inference input has a different unit of work. The same fragment may need to appear in many repositories, composed with local text, constrained to a version range, and verified as a span inside a file those tools already track.

Nested-project features pin whole trees or paths. Git submodules record a gitlink to another repository commit. Mercurial subrepos record nested state in `.hgsubstate`. Subversion `svn:externals` maps a local path to a URL and ideally a revision; since 1.6 a file external can pull one whole file from the same repository. CVS modules maps checkout names to directories or files inside one CVSROOT. Git subtree copies another project's tree into a subdirectory of this repository.

Prayfile resolves a version range across packages, composes named exports into one `AGENTS.md`, reconstructs and checksums a managed span, and uses the same composition recipe on a Git working tree or an SVN working tree.

That is the same split Bundler and Cargo already make with Git. A gem or crate can live in a Git repository; `Gemfile.lock` still records `revision`, and `Cargo.lock` still records the git commit, so later installs fetch that revision even if the branch moved. `Prayfile.lock` plays that role for inference input. Git remains a source and a history store. `Prayfile.lock` is the resolver record.

`pray publish` and `pray sync` can record updates through Git, Mercurial, or a configured command backend. Git and Mercurial working trees are auto-detected for that recording. Subversion and CVS record through the command backend when a team needs it.

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
* can install packages directly from git repositories when they are used as distribution roots
* can record `publish` and `sync` updates through Git, Mercurial, or configured command backends, with optional remote push when enabled
* can scaffold a Prayfile, a local prayer, and a distribution root through `pray init`, `pray prayer init`, and `pray repo init`. Repository shapes: [docs/repository-layouts.md](docs/repository-layouts.md)
* enables reviewable updates through normal version control workflows
* avoids arbitrary package code execution

## Operating model

Prayfile combines Bundler-style dependency declaration, resolution, and locking with Terraform-style planning, materialization, and drift detection. The [core model](#core-model) names the artifacts used at each step.

```text
Prayfile        declares desired input dependencies
Prayfile.lock   records exact resolved state
cache           preserves original source fragments
renderer        formats target-specific files
target files    affect inference when loaded by tools
```

The lockfile is the authoritative record of accepted input. The cache preserves material for recovery, and rendered files carry the selected input to consuming tools.

## Repository model

Projects can consume Pray packages through the manifest, lockfile, and rendered target files.

The usual repository layout is:

```text
Prayfile
Prayfile.lock
AGENTS.md
CLAUDE.md
.github/copilot-instructions.md

.pray/           # ignored by default
.pray/vendor/    # optional, committed only for hermetic/offline mode
```

A project that authors or publishes prayers uses `prayers/` as described in [repository layouts](docs/repository-layouts.md).

`Prayfile` is committed because it declares desired input dependencies.

`Prayfile.lock` is committed because it records the exact resolved state.

Rendered target files are usually committed so consuming tools can read the selected input from the repository.

`examples/` contains small, self-contained project trees that show different Prayfile layouts and rendering styles.

The local `.pray/` directory is ignored by default. Vendor copies under `.pray/vendor/` may be committed in hermetic, regulated, air-gapped, or long-term preservation modes.

Expanded packages remain in the local dependency cache by default.

## Materialization and effect

Only material that is rendered, expanded, or otherwise loaded into the inference input should be assumed to affect inference directly.

A package reference, bookmark, source path, import marker, or lockfile entry may affect discovery, routing, retrieval, or attention if the consuming tool reads it. The referenced content affects inference directly when the tool loads that content.

Prayfile’s default assumption is conservative: rendered target files are the primary inference-facing artifacts.

## Prayer and silence

Prayer is the selected language rendered before inference. Silence is the deliberate exclusion of resolved material from rendered input.

Authors select words, rules, and formatting for each target. Silence lets them exclude irrelevant fragments and source metadata from that target's input.

Words, repetition, formatting, and deliberate omissions all form part of the input presented to a model. Authors choose which dependencies contribute to each target and which stay out of it. This makes silence an explicit part of composition.

Silence must be explicit and reproducible. If a package contains fragments that are resolved but excluded from a target, that exclusion should be recorded in `Prayfile.lock`.

## Philosophy

Inference input is the language placed before inference: the surrounding material that affects what a model notices, ignores, repeats, refuses, prioritizes, imitates, formats, or treats as important.

This includes explicit instructions, examples, checklists, project policies, workflow notes, style guides, tool descriptions, memories, conventions, file names, headings, ordering, repetition, and local vocabulary.

Input can be declarative, procedural, stylistic, or inherited from previous work. Each form becomes part of the conditions under which inference happens, which makes input operational.

If input can shape output, then input should be inspectable. If it can drift, then it should be locked. If it can be shared, then it should have provenance. If it can disappear upstream, then it should be recoverable. If formatting can change behaviour, then rendered output should be reproducible.

Prayfile exists because the language around an inference engine is already infrastructure.

Prayfile treats that material as a dependency graph so projects can reproduce the selected input. Inference output remains probabilistic and depends on the provider, model, tool, and situation.

## Why the name Prayfile?

The name uses prayer as an analogy for language placed before an uncertain response, intended to shape attention and action. In this project, the analogy refers to how authors select, order, format, repeat, and omit input.

The alternatives emphasize different parts of the design:

* `Agentfile` suggests ownership by an agent runtime. Prayfile packages input for coding assistants, chat systems, review tools, documentation generators, local inference wrappers, automation tools, and future consumers.
* `Cookfile` suggests recipes and execution. Prayfile's operations are dependency resolution and rendering of input data.
* `Mantrafile` emphasizes repetition and stable phrasing. Prayfile also needs to express version locking, provenance, recovery, formatting, and composition from multiple sources.

For this project, the prayer analogy includes language used for orientation: asking, praising, confessing, remembering, surrendering, repeating, listening, and preparing for a response. It draws attention to form as well as semantic content, including rhythm, silence, shared context, expectation, and attention.

The technical purpose is to make the selected language, its form, and its provenance auditable. The intended influence includes judgment, refusal, style, formatting, and action.

## Does Prayfile execute package code?

No.

Prayfile packages are data. Resolution and rendering are deterministic and do not require executing arbitrary package code.

This keeps package use auditable and reduces the security risks of distributing executable tooling alongside inference input material.

## Distribution points

Prayfile packages may be fetched from distribution points.

A distribution point is a registry-like source for package metadata, package archives, signatures, checksums, derived metadata, confessions, usage feedback, and optional web documentation.

Deployment notes for `pray serve` on Heroku, Fly.io, Hetzner, Cloudflare, and Dokku live in `docs/serve-platforms.md`. Git-free static hosting (Cloudflare Pages, R2, S3) lives in `docs/static-distribution.md`.

An example public or private distribution point could be:

```text
https://prayers.kisko.dev
```

The static registry layout and metadata fields are defined in [RFC 0060](rfcs/0060-distribution.md#29-static-registry-protocol).

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

Prayfile should also support direct sources such as local paths, git repositories, archive URLs, and vendored `.praypkg` files. A centralized distribution point is useful, but not required.

Pray packages may consist only of editable text files and the required `*.prayspec`. Distribution points may compute annotations such as summaries, topics, counts, and embeddings while leaving the artifact hash, tree hash, and version identity unchanged.

Package metadata, summaries, scores, confessions, and derived annotations remain claims until verified. Any participant may produce them through manual review, hardcoded logic, deterministic heuristics, local inference, cloud inference, or generative models. Their method of production leaves the verification requirements unchanged.

Packages and rendered bytes are verified separately: package hashes and signatures cover the accepted artifact, while render digests or equivalent deterministic checks cover the final injected bytes. Lockfile verification records can bind package claims, render plans, rendered bytes, and confessions to hashes and provenance. [RFC 0060](rfcs/0060-distribution.md#294-zero-trust-verification-and-engine-agnostic-annotations) defines these boundaries and annotation provenance.

Prayfile should also leave room for peer-to-peer distribution transport: torrent-style seeding and collective-network discovery through a DHT, inspired by systems like BitTorrent, Freenet, and GNUnet. That transport should preserve the same hash verification, signature checking, and provenance guarantees as static hosting.

Distribution points may also form federated networks through server-to-server (S2S) synchronization. Servers can establish explicit peer relationships and sync package metadata and artifacts, similar to FIDONet-style store-and-forward networks. Federation allows distribution points to mirror packages, share availability across the network, and maintain eventual consistency while preserving signatures and provenance. Clients remain unaware of federation and simply query a single server.

## Publishing and signatures

Publishing changes the shared material placed before inference, so publishing must be treated as a high-trust operation.

A hardened distribution point should require:

* account authentication
* two-factor authentication
* passkey support and an explicit passkey check on publishing
* a package signature on publish, with SSH signing key support
* publisher identity recorded in package metadata
* immutable package archives after publish
* yanking or deprecation instead of silent replacement
* an append-only audit log for publish events
* checksum verification before and after upload

The security baseline and signature format are defined in [RFC 0050](rfcs/0050-security-and-trust.md). [RFC 0061](rfcs/0061-publish-version-preservation.md) defines version preservation, including the handling of prior yanks.

`pray publish` should create the package archive, compute content hashes, sign the package, and upload the package plus metadata to the selected distribution point.

`pray install`, `pray update`, and `pray verify` should verify package hashes and signatures according to policy.

The package archive is the object being distributed. `Prayfile.lock` records what the project accepted, including the expected archive, source fragments, publisher claim, and signature policy.

Ed25519 signatures verify that a key signed the package hashes. Identifying a publisher also requires a trusted binding to that key.

## Confession feedback

A confession is signed acceptance or rejection feedback about a specific prayer package, version, fragment, rendered span, or resolved lockfile entry. `pray confess` submits that feedback with an optional free-form note:

```sh
pray confess amkisko/engineering-audit --version 2.11.0 --accepted
pray confess amkisko/engineering-audit --version 2.11.0 --rejected --note "Too broad for small maintenance tasks."
pray confess --from-lock p7f3k9m2 --accepted --note "Useful authorization checklist."
```

The record attests to acceptance or rejection in a specific usage context. Inference results, benchmark results, review scores, and package ratings need their own evidence.

Distribution points may aggregate confessions to show package usefulness, rejection patterns, compatibility issues, version regressions, or fragment-level feedback.

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

Confessions should be append-only. Submit corrections as new confessions and preserve the original records.

## `pray serve`

`pray serve` runs a local or self-hosted distribution point using the same package, metadata, verification, and feedback contracts described above.

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

`pray repo init` creates that `./prayers` distribution root in the current workspace. You can publish to it directly, or point a git source at the repository and let `pray install` discover the committed distribution tree.

### Local serve behind Cloudflare Zero Trust Tunnel

A common way to distribute prayers from a laptop or private server is to keep `pray serve` bound to localhost and publish it through a Cloudflare Zero Trust Tunnel.

1. Authenticate Cloudflare access on the machine that runs `pray serve`.
2. Create a named tunnel.
3. Route a hostname to the local `pray serve` port.
4. Add Cloudflare Access if you want login protection before people can reach the server.

Example setup:

```sh
cloudflared tunnel login
cloudflared tunnel create pray
cloudflared tunnel route dns pray prayers.example.com
```

Example tunnel config:

```yml
tunnel: <tunnel-uuid>
credentials-file: ~/.cloudflared/<tunnel-uuid>.json

ingress:
  - hostname: prayers.example.com
    service: http://127.0.0.1:7429
  - service: http_status:404
```

Run the tunnel:

```sh
cloudflared tunnel run pray
```

For a quick one-off test, Cloudflare also supports a temporary tunnel URL:

```sh
cloudflared tunnel --url http://127.0.0.1:7429
```

Keep publish and moderation routes behind Cloudflare Access or another stronger auth layer when those actions are exposed publicly.

Cloudflare Access in front of GET `/v1/` is a browser login. `pray install` does not send Access service-token headers or complete an identity-provider redirect. For a private registry on the public internet, use an overlay (Tailscale Serve or WireGuard) or `pray+ssh`. Do not expect Access IdP login to unlock install.

## Bundled assets

The Rust reference implementation should not bundle binary assets.

No fonts, images, icons, databases, compressed binary blobs, or opaque static bundles should be embedded into the executable.

Allowed embedded assets:

* HTML templates
* CSS
* plain text help
* plain text schema examples
* small text fixtures required for tests or diagnostics

Avoiding bundled binary assets keeps the executable inspectable, reproducible, portable, and easier to package across platforms.

The built-in web interface, if provided, should be minimal and text-first.

## How does Prayfile keep input consistent?

`Prayfile.lock` records the exact resolved dependency graph, selected versions, source references, content hashes, silenced fragments, render targets, renderer versions, formatting options, signature metadata, distribution point metadata, confession references, provenance metadata, and managed span records.

Each managed span (a prayer rendered between pray markers) has a lockfile entry that stores:

* marker ID
* target file path
* ideal checksum : semantic hash of the expected managed body (content between markers, excluding pray comment lines)
* line positions : opening and closing marker line numbers in the target file
* provenance : package, export, source fragment checksum, silenced flag, and related metadata

The lockfile is the authoritative record of what should appear in each managed span and where it should live. These records let projects verify that rendered files still match the resolved state and trace each managed span to its source fragments.

A local compressed cache may store the original source fragments used during resolution. This gives the project a recoverable copy of the resolved input material, independent of mutable upstream repositories, registries, or URLs.

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

## Render markers

Rendered target files should not duplicate the dependency graph, package list, source URLs, source hashes, or provenance records already stored in `Prayfile.lock`.

Rendered files use compact citation markers to identify managed spans and look up their provenance records in `Prayfile.lock`.

Markdown targets use this canonical marker form:

```md
<!-- pray:p7f3k9m2 -->

...rendered content...

<!-- pray:p7f3k9m2 -->
```

The same marker appears exactly twice on its own lines: first to open the block, then to close it. Nested blocks and unmatched markers are invalid.

A marker ID must be opaque. It must not encode package names, topic names, versions, hashes, source paths, or semantic labels.

Marker IDs must use only lowercase ASCII letters and digits.

Marker IDs should be 8-16 characters.

The purpose of a marker is region identity, drift detection, and lockfile lookup.

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

The [render markers](#render-markers) section shows the canonical Markdown form and ID restrictions.

## Planning, verifying, applying, and drift

Prayfile separates detection from materialization.

### `pray plan`

Computes the changes that would happen to resolved packages, lockfile entries, cache contents, and rendered target files.

### `pray apply`

Materializes the planned changes.

After render, `pray apply` refreshes `Prayfile.lock`:

* updates ideal checksums for each managed span
* updates opening and closing marker line positions
* adds, updates, or removes managed span records when prayers are introduced, re-rendered, or silenced

`pray install` and `pray render` also write dest and `Prayfile.lock`. `pray plan` is the dry-run of those writes.

### `pray verify`

Read-only consistency check against the current lockfile and on-disk target files.

For every managed span record, `pray verify` locates the marker pair in the target file and checks:

* the marker pair is present; a missing pair reports a removed prayer
* the managed body matches the lockfile ideal checksum; changed content reports a custom implementation
* the marker line positions match the lockfile positions; moved markers report position drift even when the content checksum matches

`pray verify` reports mismatches without modifying `Prayfile.lock` or target files.

### `pray drift`

`pray drift` reports everything [`pray verify`](#pray-verify) reports, plus:

* renderer drift : on-disk file matches the lock, but a fresh render from current packages and `Prayfile` would produce different ideal checksums or different prayers
* orphan marker : a pray marker pair exists in a target file with no matching lockfile managed span record

`pray drift` is for review and CI. It does not refresh the lockfile.

To accept intentional changes after plan review, run `pray apply`.

### `pray render`

Writes dest and `Prayfile.lock` from current inputs. `--check` writes nothing and fails unless dest already matches that fresh render. The dry-run of a write is `pray plan`.

### Other commands

Example command set:

```sh
pray init
pray prayer init
pray repo init
pray add amkisko/engineering-audit
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

Select a project root, manifest path, or render environment before any subcommand:

```sh
pray --path /path/to/project --env development install
pray --path /path/to/project --file-path configs/Prayfile plan
```

Equivalent environment variables are `PRAY_PATH`, `PRAY_FILE_PATH`, and `PRAY_ENV`. The reference CLI also reads those keys from the project `.env` without overriding process values already set.

A home directory can be the project root. It is a normal project folder: `pray` may write any declared destination under that root.

```sh
pray --path "$HOME" install
```

`Prayfile.lock` and `.pray` live in that root. Exclusive `file: ".zshrc"` writes unmarked bytes. Do not write `file: "~/.zshrc"`; a leading tilde is rejected as a destination string, not expanded as home. Compose uses HTML comments, so compose of `.zshrc` or `.json` fails and names `file:` as the unmarked path. Prefer a native include of the unmarked file. Prefer `pray trust set-require-signed-packages` for remote packages. The CLI does not walk parent directories for a Prayfile.

`--path /` with `/Prayfile` is allowed. Choose a narrower root to limit writes: every declared destination is relative to the selected root.

Group blocks in `Prayfile` assign environment names to packages. Ungrouped packages always render. With no environment selected, only ungrouped packages render. With `--env development`, ungrouped packages plus packages in the `development` group render. Resolution and locking still include every declared package.

## Package storage modes

Most repositories use the default [repository model](#repository-model), with packages fetched from sources and cached locally.

Hermetic mode:

```text
Prayfile
Prayfile.lock
AGENTS.md
.pray/vendor/*.praypkg
.gitignore: .pray/cache/, .pray/write-state/, .pray/state.json
```

Use this for air-gapped repositories, regulated work, long-term archival, or cases where upstream availability cannot be trusted.

Generated-only mode commits the manifest and lockfile while generating target files on demand. Use it only when every developer, CI worker, and inference tool reliably runs `pray render` before use.

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

## Scope and responsibility

Prayfile manages pre-inference material as data across model providers and consuming tools. Maintainers choose and review instructions and evaluate their effects in each project. Inference execution remains the consuming project's responsibility.

Packaging and verification establish input integrity; they do not guarantee better model behaviour. Package execution is prohibited as described [above](#does-prayfile-execute-package-code).

## Core model

| Concept                | Role                                                                                                  |
| ---------------------- | ----------------------------------------------------------------------------------------------------- |
| `Prayfile`             | Human-authored input dependency manifest                                                              |
| `Prayfile.lock`        | Machine-authored resolved state, including ideal checksums and marker line positions per managed span |
| managed span           | Rendered region between matching pray markers, tracked by a lockfile record                           |
| ideal checksum         | Semantic hash of expected managed span body stored in `Prayfile.lock`                                 |
| `*.prayspec`           | Package definition                                                                                    |
| `*.praypkg`            | Package archive                                                                                       |
| distribution point     | Registry-like source for packages, metadata, checksums, signatures, feedback, and docs                |
| package signature      | Proof that a signing key approved package hashes; publisher identity needs a separate trust binding   |
| confession             | Signed acceptance or rejection feedback for a resolved prayer                                         |
| network fingerprint    | Additional verification signal attached to a signed confession                                        |
| local cache            | Compressed storage for original resolved source fragments                                             |
| local vendor directory | Optional committed package cache                                                                      |
| rendered target file   | Inference-facing materialized output                                                                  |
| pray marker            | Compact citation into the lockfile                                                                    |
| silenced fragment      | Resolved fragment intentionally excluded from target output                                           |
| `pray`                 | Reference CLI                                                                                         |
| `pray serve`           | Built-in distribution point server                                                                    |

## Design principles

These principles summarize the project for readers of this overview. [RFC 0002](rfcs/0002-problem-and-positioning.md#2-core-positioning) records them in the specification.

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
| `rfcs/`          | Normative specification (RFC 0111)      |
| `AGENTS.md`      | Contributor and inference tool workflow |
| `spec/README.md` | Test coverage guidelines                |

## Read the specification

Start with the [RFC index](rfcs/README.md). The main contracts are:

* positioning and design principles: [RFC 0002](rfcs/0002-problem-and-positioning.md)
* manifest and package formats: [RFC 0010](rfcs/0010-core-formats.md), [RFC 0011](rfcs/0011-prayspec-and-package.md)
* resolution, lockfiles, and managed span records: [RFC 0020](rfcs/0020-resolve-and-lock.md)
* rendering, markers, verification, and output ownership: [RFC 0030](rfcs/0030-render-markers-ownership.md), [RFC 0031](rfcs/0031-ownership-and-generated-output.md)
* CLI commands, formatting, and plan/apply: [RFC 0040](rfcs/0040-cli-surface.md)
* checksums, publishing, signing, and feedback policy: [RFC 0050](rfcs/0050-security-and-trust.md)
* distribution points and registry metadata: [RFC 0060](rfcs/0060-distribution.md)
* cache layout and project-local storage: [RFC 0070](rfcs/0070-reference-implementation.md), [RFC 0071](rfcs/0071-project-local-pray-directory.md)

## Additional packaging

### npm (Node.js)

Install the TypeScript `pray` CLI from [npm](https://www.npmjs.com/package/pray-cli) (depends on [`pray-core`](https://www.npmjs.com/package/pray-core)):

```sh
npm install -g pray-cli
```

Or run without a global install:

```sh
npx pray-cli install
```

The CLI lives in `npmjs/pray-cli`. The library lives in `npmjs/pray-core`.

### TypeScript library (npm / jsr.io)

```sh
npm install pray-core
# or
npx jsr add @kiskolabs/pray-core
```

```ts
import { parseManifest } from "pray-core";
// or: import { parseManifest } from "@kiskolabs/pray-core";
```

JSR ships TypeScript source for `@kiskolabs/pray-core`. The Node CLI binary remains the npm `pray-cli` package. Operators use `scripts/release/jsr.sh` for the library and `scripts/release/npm.sh` for core then CLI.

### Nix

From a checkout or from GitHub once the flake is on the default branch:

```sh
nix run .#pray-cli -- version
nix run github:kiskolabs/pray -- version
```

### AUR (Arch Linux)

The in-tree recipe is `packaging/aur/pray-cli/PKGBUILD`. It is not on the AUR until a maintainer adopts it. After the GitHub tag exists, refresh checksums with `make bump-aur`. Other Linux packaging notes: [packaging/README.md](packaging/README.md).

### RubyGems (Ruby)

Install the Ruby `pray` CLI from [RubyGems](https://rubygems.org/gems/pray-cli):

```sh
gem install pray-cli
```

Or from this repository:

```sh
cd rubygems/pray-cli
bundle install
bundle exec pray version
```

The gem lives in `rubygems/pray-cli` and provides the same core workflow: parse `Prayfile`, resolve local and registry packages, write `Prayfile.lock`, render targets, and verify drift.

## Contributing

Bug reports, design discussions, examples, and pull requests are welcome.

Please read `CONTRIBUTING.md` before submitting changes.

The specification is currently the primary area of development and feedback.

## Security

Please do not disclose security vulnerabilities through public issues.

See `SECURITY.md` for responsible disclosure instructions.

## Links

- [GitHub](https://github.com/kiskolabs/pray)
- [GitLab](https://gitlab.com/amkisko/pray)
- [crates.io](https://crates.io/crates/pray-cli)
- [npm pray-cli](https://www.npmjs.com/package/pray-cli) · [npm pray-core](https://www.npmjs.com/package/pray-core)
- [jsr.io @kiskolabs/pray-core](https://jsr.io/@kiskolabs/pray-core)
- [RubyGems](https://rubygems.org/gems/pray-cli)
- [libraries.io](https://libraries.io/cargo/pray-cli)
- [Deps.dev](https://deps.dev/cargo/pray-cli)
- [SonarCloud](https://sonarcloud.io/project/overview?id=kiskolabs_pray)
- [Snyk](https://snyk.io/test/github/kiskolabs/pray)
- [Codecov](https://app.codecov.io/github/kiskolabs/pray)
- [OpenSSF Scorecard](https://scorecard.dev/viewer/?uri=github.com/kiskolabs/pray)

## License

MIT. See `LICENSE.md`.
