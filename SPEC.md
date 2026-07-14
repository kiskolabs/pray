# Prayfile Open Specification

**Status:** Active development v0.1  
**Primary file names:** Prayfile, Prayfile.lock, *.prayspec, *.praypkg  
**Reference CLI name:** pray  
**Project name:** pray  
**Reference implementation target:** systems language  
**Specification goal:** language-independent, platform-independent, implementation-independent

---

## 1. Summary

Prayfile is an open specification for reproducible pre-inference input composition.

It lets projects declare shared instructions, policies, memories, templates, review checklists, formatting rules, and workflows in one place; resolve them deterministically; lock exact versions and hashes; preserve original source fragments; and render tool-specific outputs with compact provenance markers.

The core model is:

| Concept | Role |
|---------|------|
| Prayfile | human-authored input dependency manifest |
| Prayfile.lock | machine-authored resolved state |
| *.prayspec | package definition |
| *.praypkg | package archive |
| distribution point | registry-like source for packages, metadata, checksums, signatures, feedback, and docs |
| pray | reference CLI |

Prayfile is conceptually similar to dependency manifest.

Prayfile.lock is conceptually similar to dependency lockfile.

*.prayspec is conceptually similar to *.packagespec.

But unlike legacy package registries, the specification must not require host-language execution. All files must be parseable as static declarations by any implementation in any language.

The goal is not to create a magic agent. The goal is to distribute, lock, verify, and render inference input cleanly—with compact pray markers that cite `Prayfile.lock`.

---

## 2. Core positioning

**One-sentence definition:**

Prayfile is an open specification for reproducible inference input composition.

**Short pitch:**

Modern tools rely on surrounding instruction files, templates, review checklists, memories, formatting rules, and workflow notes. These files are often distributed manually through copy-paste. Prayfile lets projects declare shared input dependencies, resolve them deterministically, lock exact versions and content hashes, preserve original source fragments, and render tool-specific outputs with compact provenance markers.

**FAQ:**

| Question | Answer |
|----------|--------|
| Is this a prompt framework? | No. The durable problem is packaging and distributing the material placed before inference—not prompt design itself. |
| What is input drift? | The gradual divergence of instructions, policies, templates, memories, formatting rules, and workflow assumptions between projects. |
| Why now? | More tools now read repository-local instruction files, but manual copy-paste still does not scale. |
| Is the spec final? | Not yet. Terminology, formats, and behaviour may still evolve as the system is hardened through real-world use. |
| Implementation status? | The specification and reference CLI evolve together, with production readiness as the goal. |

**Design principles:**

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
Keep revision history visible through the configured repository backend.
Never execute package code.
Never hide updates.
Keep diffs small.
Preserve provenance.
Support rollback.
Respect silence.
Avoid bundled binary assets.
```

### Core values

Inference input is operational—it shapes what models notice, ignore, repeat, refuse, prioritize, imitate, format, or treat as important. Prayfile treats observability and trust as first-class requirements, not optional polish.

| Value | What it means |
|-------|---------------|
| Auditable traces | Every managed rendered span carries a compact pray marker. `Prayfile.lock` records exact resolved state, ideal checksums, marker line positions, source checksums, silenced fragments, and provenance metadata. |
| Temporal clarity | Lockfile and drift semantics show what changed between resolves. Version control carries when. Pray markers enable surgical rollback, blame, and review without rereading entire target files. |
| Measurable effects | Effects are measured at the dependency boundary first: manifest → lock → rendered bytes → reviewable diff. Inference behaviour remains human-validated; the specification does not score model quality. |
| Security | Input packages are supply-chain inputs: static declarations only, hash-verified, path-safe, explicitly updated, optionally signed. Audit trails align with integrity—implementations can prove what was installed, from where, and at which version. |

These values inform lockfile fields, pray markers, `pray drift` output, `pray verify` checks, and the security model in later sections.

### Production intent

Packaging shapes, tool conventions, and workflow surfaces for inference input will keep changing drastically. Prayfile is designed to stay useful while that surface changes by defining stable contracts, clear indicators, and reviewable change paths.

To observe change, you need indicators. Prayfile defines them as contracts: pinned lock state, pray markers, explicit diffs, integrity checks, and signed feedback. The core values above are those indicators made normative so teams can measure what altered, when, and from where while the surrounding ecosystem shifts.

---

## 3. Problem

Inference-oriented development now commonly uses files and folders such as:

- instruction files
- instruction libraries
- prompt templates
- review checklists
- memories
- formatting rules
- workflow notes

Teams copy the same context between repositories.

This causes:

- duplicated instructions
- stale context
- large noisy diffs
- manual copy-paste updates
- unclear package ownership
- hidden behavioral drift
- hard rollback
- hard audit
- conflicting rules
- different output for different inference tools
- accidental private-input leakage
- giant merged instruction files

Inference input is not passive documentation. It affects what models notice, ignore, repeat, refuse, prioritize, imitate, format, or treat as important.

Therefore it should be treated as a dependency.

---

## 4. Goals

The specification prioritizes:

- human-readable files
- small git diffs
- minimal generated output
- deterministic installs
- deterministic rendering
- explicit updates
- lockfile-based reproducibility
- cross-platform behavior
- implementation in any language
- support for public/private/local and peer-to-peer distribution
- safe package installation
- no arbitrary code execution
- easy recovery
- easy vendoring
- easy CI validation
- clear package ownership
- tool-neutral package model
- tool-specific adapters
- auditable provenance for every managed rendered span
- temporal impact visible through lockfile and diff semantics
- measurable textual effects at the manifest–lock–render boundary
- supply-chain security for context packages

The main aesthetic is:

```
less formatting religion
more stable meaning
less generated sludge
more readable reviewable context
```

---

## 5. Non-goals

This system is not:

- an agent runtime
- a chat memory system
- a session-end learning hook
- a prompt-injection firewall
- a secret manager
- a policy/governance platform
- a marketplace ranking system
- a background self-updater
- a hidden instruction mutator
- an autonomous context writer
- a replacement for human review
- a package manager for executable code
- a product built around any single agent workflow shape (for example, today's skills packaging)
- a bet that current inference-input conventions will stay unchanged; it assumes they will evolve while reproducible composition remains

Self-recovery means deterministic reconstruction from Prayfile.lock.

Self-update means explicit `pray update`.

It must not mean hidden mutation.

---

## 6. Naming

Recommended names:

| Concept | Name |
|---------|------|
| Spec / manifest | Prayfile |
| Lockfile | Prayfile.lock |
| Package spec | *.prayspec |
| Package archive | *.praypkg |
| Distribution point | registry-like source for packages, metadata, checksums, signatures, feedback, and docs |
| CLI | pray |
| Project / implementation | pray |
| Implementation crate/package | pray |

**Prayfile** and **Prayfile.lock** are the manifest and lockfile names.

The reference CLI is **pray**. See `README.md` for project positioning and name rationale.

Example command usage:

```
pray init
pray prayer init
pray repo init
pray add sample/webapp "~> 2.1"
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

When a distribution repository lives inside a larger checkout, the recommended root folder is `prayers/` (for example, created by `pray repo init`).

Implementations may split resolve and render internally. Those phase names are not CLI commands or aliases.

Semantic analogy:

| Prayfile concept | Analogy |
|-------------------|---------|
| Prayfile | recipe |
| Prayfile.lock | exact brew record |
| package | ingredient / volume |
| export | portion |
| distribution point | pantry |
| install | resolve + render |
| update | re-resolve |
| render | materialize |
| verify | taste test |
| vendor | jar on the shelf |

---

## 7. Ecosystem analogy

| Reference package ecosystem | Prayfile ecosystem |
|----------------|---------------------|
| dependency manifest | Prayfile |
| dependency lockfile | Prayfile.lock |
| *.packagespec | *.prayspec |
| .legacy-archive | .praypkg |
| resolver install | pray install |
| resolver update | pray update |
| package build | pray package |
| package publish | pray publish |
| legacy package registry | distribution point / static index |

**Important difference:**

Legacy registries may execute host-language code.  
Prayfile must parse declarations only.

### Dependency ecosystem alignment

Prayfile is lockfile-shaped for resolve and render, with an additional materialize phase. Existing dependency ecosystems provide the closest reference patterns; the core values in section 2 extend their indicator model to inference-input dependencies.

| Prayfile | Dependency ecosystem |
|----------|---------------------|
| Prayfile | dependency manifest |
| Prayfile.lock | lockfile |
| *.prayspec | package spec |
| *.praypkg | package archive |
| resolve (lock) | resolver / lock step |
| render (fetch + materialize) | no direct equivalent — dependencies install as their own artifacts, not merged context files |
| pray verify | checksum / integrity checks |
| pray drift | lockfile diff plus rendered-output diff |

| Core value | Dependency ecosystem | Prayfile extension |
|------------|---------------------|---------------------|
| Auditable traces | lockfile pins; versioned artifacts | compact pray markers inside rendered target files |
| Temporal clarity | lockfile history; explicit updates | `pray drift` across lock and render; marker-level blame and rollback |
| Measurable effects | manifest → lock → install; behavior validated by tests | manifest → lock → rendered bytes → diff; inference behaviour stays human-validated |
| Security | checksums; optional signing; vendoring | same supply-chain baseline; packages are static declarations only — no host-language execution |

Prayfile does not replace dependency ecosystems. It applies reproducibility and audit patterns proven necessary for code dependencies to inference-input dependencies: lock what resolved, render what landed, cite managed spans compactly in target files.

A planned host-language adapter may provide runtime loading and assembly for applications that need it; it consumes `Prayfile.lock` and cache artifacts produced by `pray`, and it does not replace the CLI resolve/render pipeline.

---

## 8. Terminology

**Prayfile** — Human-authored dependency manifest.

**Prayfile.lock** — Machine-authored exact resolved state.

**prayspec** — Package definition file.

**agent package** — Versioned bundle of agent-context content.

**export** — Named unit provided by a package.

Examples: `webapp-review`, `testing-guidance`, `security-basics`, `ui-components`, `incident-template`

**target** — An agent tool or output environment.

Examples: `tool_a`, `tool_b`, `tool_c`, `tool_d`, `tool_e`, `generic`

**adapter** — Mapping from generic package exports into target-specific files.

**render** — Process of creating actual target files from locked package state.

**managed file** — Generated file owned by pray.

**local file** — Human-owned project file included or appended into rendered output.

**source** — Place where packages are resolved from.

Examples: registry, static index, git, local path, tarball, OCI artifact, file share

**frozen install** — Install mode that refuses to update lockfile or generated files.

**annotation** — Untrusted derived metadata, confession, or analysis output that describes a package, render, or usage event.

**claim** — Any annotation, summary, confession, score, or metadata field supplied by a client, server, or engine.

**render digest** — Exact hash of the final injected bytes after render and normalization.

---

## 9. Repository layout

### Recommended project layout:

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

## 10. Commit policy

**Recommended default for most repositories:**

- commit Prayfile
- commit Prayfile.lock
- commit rendered target files when tools require repository-local files
- ignore cache
- ignore state

**Recommended for local personal context:**

- commit Prayfile
- optionally commit Prayfile.lock
- ignore generated local tool output
- ignore cache
- ignore state

**Recommended for offline / archival workflows:**

- commit Prayfile
- commit Prayfile.lock
- commit `.pray/vendor`
- commit generated files if target tools need them

---

## 11. Prayfile design

Prayfile is a declarative declarative manifest DSL.

It must be:

- human-readable
- dependency-manifest-like
- static
- non-executable
- parseable by any implementation
- convertible to canonical data model

It must not be executable host language.

Allowed style:

```manifest
prayfile "1"
source "default", "https://agents.example.com"
target :tool_a do
  output "INSTRUCTIONS.md"
  skills ".agents/skills"
end
agent "sample/base", "~> 1.4",
  exports: ["testing-basics", "security-basics"]
local ".agents/project.md"
render mode: :managed
```

Forbidden:

```manifest
if ENV["X"]
end
require "network/client"
File.read("secret.txt")
system("curl ...")
eval("...")
```

The parser must reject:

- conditionals
- loops
- variable assignment
- method calls outside the DSL
- manifest constants outside allowed root objects
- file reads except declared local paths
- environment interpolation
- shell execution
- network access
- dynamic evaluation

### Editor language mode

`Prayfile` and `Prayfile.lock` have no file extension. Editors that auto-detect language from buffer content may misclassify them (for example as TypeScript) because the DSL uses Gemfile-like keyword arguments, string literals, and symbol-like tokens (`:managed`). That is an editor integration gap, not invalid Prayfile syntax.

Until a dedicated `prayfile` grammar exists, pin language mode by filename in editor settings.

Ruby is the closest practical match for `Prayfile` highlighting today:

```json
"files.associations": {
  "Prayfile": "ruby",
  "Prayfile.lock": "toml"
}
```

Use plain text when syntax highlighting is not needed and false diagnostics are distracting:

```json
"files.associations": {
  "Prayfile": "plaintext",
  "Prayfile.lock": "toml"
}
```

When extensionless files keep flipping language after auto-detection, disable detection workspace-wide:

```json
"workbench.editor.languageDetection": false
```

Manual override through the status bar language picker (Ruby or Plain Text) should stick once chosen; auto-detection must not override an explicit picker selection.

Longer term, a small editor extension should register language id `prayfile` with a TextMate grammar and ship `files.associations` for `Prayfile` and `Prayfile.lock`, similar to how `Gemfile` is commonly associated with Ruby or handled by dedicated tooling.

Implementations may document the snippets above in repository `README.md` or contributor setup notes; the spec does not require a particular editor.

---

## 12. Canonical manifest model

Every valid Prayfile compiles to a canonical language-neutral model.

Example:

```json
{
  "prayfile_version": "1",
  "sources": [
    {
      "name": "default",
      "kind": "registry",
      "url": "https://agents.example.com"
    }
  ],
  "targets": [
    {
      "name": "tool_a",
      "outputs": ["INSTRUCTIONS.md"],
      "skills": [".agents/skills"]
    }
  ],
  "packages": [
    {
      "name": "sample/base",
      "constraint": "~> 1.4",
      "source": "default",
      "exports": ["testing-basics", "security-basics"]
    }
  ],
  "local": [
    {
      "path": ".agents/project.md",
      "position": "after"
    }
  ],
  "render": {
    "mode": "managed",
    "conflict": "fail",
    "churn": "minimal"
  }
}
```

The canonical model, not textual formatting, defines meaning.

Whitespace-only changes should not affect lockfile resolution.

---

## 13. Minimal Prayfile example

```manifest
prayfile "1"
source "default", "https://agents.example.com"
target :tool_a do
  output "INSTRUCTIONS.md"
  skills ".agents/skills"
end
target :tool_b do
  output "TOOL_B.md"
  skills ".tool-b/skills"
end
agent "public/base", "~> 1.0",
  exports: ["repository-basics", "testing-basics"]
agent "public/webapp", "~> 2.2",
  exports: ["webapp-review", "data-layer", "testing"]
local ".agents/project.md"
render mode: :managed,
  conflict: :fail,
  churn: :minimal
```

---

## 14. Larger Prayfile example

```manifest
prayfile "1"
source "default", "https://agents.example.com"
source "sample", "git+ssh://git@example.com/agent-context/index.git"
target :tool_a do
  output "INSTRUCTIONS.md"
  skills ".agents/skills"
  max_bytes 120_000
end
target :tool_b do
  output "TOOL_B.md"
  skills ".tool-b/skills"
  max_bytes 120_000
end
target :tool_c do
  rules ".tool-c/rules"
end
group :base do
  agent "sample/base", "~> 1.4",
    source: :sample,
    exports: [
      "working-agreements",
      "testing-basics",
      "security-basics"
    ]
end
group :webapp do
  agent "sample/webapp", "~> 2.1",
    source: :sample,
    exports: [
      "webapp-review",
      "data-layer",
      "testing",
      "live-pages"
    ]
  agent "public/ui-kit", "^1.0",
    exports: ["component-guidance"]
end
local ".agents/project.md", position: :after
local ".agents/testing.md", position: :after
render mode: :managed,
  conflict: :fail,
  churn: :minimal,
  header: true,
  section_markers: true
```

---

## 15. Prayfile statements

### prayfile

Declares spec version.

```
prayfile "1"
```

Required.

### source

Declares package source.

```
source "default", "https://agents.example.com"
source "sample", "git+ssh://git@example.com/agent-index.git"
source "team", "pray+ssh://pray@prayers.internal"
source "local", path: "../agent-packages"
```

Source names must be unique.

Supported source kinds: `registry`, `static_index`, `git`, `path`, `tarball`, `oci`, `pray_ssh`

### target

Declares rendered target.

```manifest
target :tool_a do
  output "INSTRUCTIONS.md"
  skills ".agents/skills"
end
```

Common target fields:

```
output "INSTRUCTIONS.md"
skills ".agents/skills"
commands ".tool-b/commands"
rules ".tool-c/rules"
max_bytes 120_000
```

Unknown target features should warn by default. Strict mode should fail.

### agent

Declares package dependency.

```manifest
agent "sample/webapp", "~> 2.1",
  exports: ["webapp-review", "testing"]
```

Supported options:

```
source: :sample
exports: [...]
targets: [...]
features: [...]
optional: true
git: "..."
tag: "..."
rev: "..."
path: "..."
tarball: "..."
oci: "..."
```

### group

Groups package declarations for environment-aware rendering.

```manifest
group :development, :test do
  agent "sample/webapp", "~> 2.1"
  agent "sample/ui-kit", "~> 1.0"
end
```

Rules:

- A group block must use `do ... end` and may list multiple environment names separated by commas.
- Only `agent` or `package` declarations are allowed inside a group block.
- Nested group blocks are rejected.
- Packages outside any group always render.
- When no render environment is selected, only ungrouped packages render.
- When `PRAY_ENV` or `--env` / `--environment` selects a name, ungrouped packages plus packages whose `groups` include that name render.
- Unknown environment names fail with the available group names.
- Group membership is part of the canonical manifest and manifest hash.
- Package resolution and lock entries remain complete for every declared package regardless of the selected environment; only rendered managed spans and provisioned files are filtered.

### local

Includes human-owned local project context.

```
local ".agents/project.md"
local ".agents/security.md", position: :after
local ".agents/private.md", optional: true
```

Supported positions: `before`, `after`, `target_after`

Default: `after`

### render

Declares render policy.

```manifest
render mode: :managed,
  conflict: :fail,
  churn: :minimal
```

Supported fields: `mode`, `conflict`, `churn`, `header`, `section_markers`, `line_endings`

---

## 16. Version constraints

Supported constraints:

```
= 1.2.3       exact
1.2.3         exact shorthand
~> 1.2        pessimistic
^1.2          compatible
>= 1.2
> 1.2
<= 2.0
< 2.0
```

Pre-release versions require explicit opt-in:

```
agent "sample/base", "2.0.0-beta.1"
```

or:

```
agent "sample/base", "~> 2.0.beta", prerelease: true
```

Default resolver should avoid pre-release versions.

---

## 17. Package names

Package names use slash-separated identifiers:

```
namespace/name
```

Examples: `public/base`, `public/webapp`, `sample/security`, `sample/testing`, `sample/ui-kit`

Valid characters: `a-z`, `0-9`, `-`, `_`, `/`, `.`

Package names are case-sensitive. Lowercase is strongly recommended.

---

## 18. Export names

Exports are named package units.

Examples: `webapp-review`, `testing`, `data-layer`, `security-basics`, `project-handoff`

Export names must be unique within a package version.

Good export names: `migration-safety`, `authorization-review`, `system-tests`, `accessibility-basics`

Bad export names: `misc`, `rules`, `all`, `stuff`, `very-important`

---

## 19. Package layout

Recommended package layout:

Packages are primarily text packages. A conforming package may consist only of minimal editable text files plus the required `*.prayspec`; richer assets are optional, not structural.

```
sample-webapp/
  sample-webapp.prayspec
  README.md
  LICENSE
  CHANGELOG.md
  exports/
    webapp-review.md
    testing.md
    data-layer.md
    live-pages.md
  skills/
    code-review/
      SKILL.md
      assets/
        checklist.md
  templates/
    pr-review.md
    incident-report.md
  adapters/
    tool_a.toml
    tool_b.toml
    tool_c.toml
```

Required: `*.prayspec`

Optional: `README.md`, `LICENSE`, `CHANGELOG.md`, `exports/`, `skills/`, `templates/`, `adapters/`, `assets/`

---

## 20. prayspec design

`*.prayspec` is the package definition file. It is inspired by legacy `.packagespec`. It is declarative but not executable host language.

Example:

```manifest
Package::Specification.new do |spec|
  spec.name = "sample/webapp"
  spec.version = "2.1.5"
  spec.summary = "web applications, testing, data layer, and live UI agent context"
  spec.description = "Shared guidance for web application review workflows, tests, migrations, and common development tasks."
  spec.authors = ["Example Maintainer"]
  spec.license = "MIT"
  spec.homepage = "https://example.com/sample/webapp"
  spec.source_code_uri = "https://vcs.example.com/sample-org/agent-packages/tree/main/sample-webapp"
  spec.changelog_uri = "https://vcs.example.com/sample-org/agent-packages/blob/main/sample-webapp/CHANGELOG.md"
  spec.pray_version = ">= 0.1"
  spec.files = [
    "README.md",
    "LICENSE",
    "CHANGELOG.md",
    "exports/webapp-review.md",
    "exports/testing.md",
    "exports/data-layer.md",
    "skills/code-review/SKILL.md",
    "adapters/tool_a.toml",
    "adapters/tool_b.toml"
  ]
  spec.exports = {
    "webapp-review" => {
      type: "fragment",
      path: "exports/webapp-review.md",
      summary: "Web application code review guidance"
    },
    "testing" => {
      type: "fragment",
      path: "exports/testing.md",
      summary: "Testing guidance"
    },
    "data-layer" => {
      type: "fragment",
      path: "exports/data-layer.md",
      summary: "Data layer guidance"
    }
  }
  spec.skills = {
    "code-review" => {
      path: "skills/code-review",
      summary: "Application code review skill"
    }
  }
  spec.templates = {
    "pr-review" => {
      path: "templates/pr-review.md",
      summary: "Pull request review template"
    }
  }
  spec.targets = ["tool_a", "tool_b", "generic"]
  spec.adapters = {
    "tool_a" => "adapters/tool_a.toml",
    "tool_b" => "adapters/tool_b.toml"
  }
  spec.add_dependency "sample/base", "~> 1.4"
  spec.metadata = {
    "prayfile.target.tool_a" => "true",
    "prayfile.target.tool_b" => "true"
  }
end
```

---

## 21. prayspec allowed grammar

Allowed:

```manifest
Package::Specification.new do |spec|
  spec.name = "..."
  spec.version = "..."
  spec.files = ["..."]
  spec.exports = { "name" => { type: "fragment", path: "..." } }
  spec.add_dependency "package/name", "~> 1.0"
end
```

Allowed value types: string, number, boolean, symbol, array, hash, nil

Allowed methods:

```
name= version= summary= description= authors= maintainers= license=
homepage= source_code_uri= changelog_uri= prayfile_version= files=
exports= skills= templates= adapters= targets= metadata=
add_dependency add_optional_dependency
```

Forbidden:

```
Dir["**/*"]
ENV["VERSION"]
require "..."
File.read(...)
system(...)
eval(...)
if ... while ... for ...
```

All files must be explicitly listed in `spec.files`. This reduces hidden package drift.

---

## 22. prayspec canonical model

Every `*.prayspec` compiles to a canonical package model:

```json
{
  "name": "sample/webapp",
  "version": "2.1.5",
  "summary": "web applications, testing, data layer, and live UI agent context",
  "license": "MIT",
  "prayfile_version": ">= 0.1",
  "files": [
    "README.md",
    "LICENSE",
    "CHANGELOG.md",
    "exports/webapp-review.md"
  ],
  "exports": {
    "webapp-review": {
      "type": "fragment",
      "path": "exports/webapp-review.md",
      "summary": "Web application code review guidance"
    }
  },
  "skills": {
    "code-review": {
      "path": "skills/code-review",
      "summary": "Application code review skill"
    }
  },
  "targets": ["tool_a", "tool_b", "generic"],
  "dependencies": [
    {
      "name": "sample/base",
      "constraint": "~> 1.4",
      "optional": false
    }
  ]
}
```

---

## 23. Export types

Supported export types:

| Type | Description |
|------|-------------|
| fragment | Text fragment rendered into a target output file |
| file | Single file provisioned into a target folder |
| folder | Directory tree provisioned into a target folder |
| template | Reusable text artifact |
| command | Tool-specific or generic command template |
| rule | Tool-specific rule file |
| asset | Static file used by a template or folder export |
| bundle | Named collection of other exports |

`skill` remains a legacy alias for `folder`.

---

## 24. Provisioned folders

A `folder` export is a directory tree copied deterministically into a target folder declared in the Prayfile.

A `file` export is a single file copied under `<target-folder>/<export-name>/`.

Example:

```ruby
target :agents do
  output "AGENTS.md"
  folder ".agents/skills"
end
```

`skills` in a target block is a legacy alias for `folder`.

Optional support files may live under `assets/`, `templates/`, or `examples/` inside the folder export.

Two packages must not install the same folder path unless conflict policy allows it.

---

## 25. Package payload rules

V1 packages are data packages.

Allowed package contents: Markdown, TOML, JSON, YAML, plain text, templates, declared assets, images/diagrams if useful for skills, scripts as inert assets only

Text files are the default package substrate; additional asset types are optional and may be omitted entirely in minimal packages.

Forbidden during install/render:

- running shell scripts
- running package hooks
- reading undeclared files
- network calls from package content
- environment-variable interpolation
- dynamic file discovery

Packages may contain executable-looking files only as inert assets. Prayfile must not execute them.

---

## 26. Package archive

Built package file: `sample-webapp-2.1.5.praypkg`

Recommended internal format: `tar.zst`

Allowed archive formats: `tar.zst`, `tar.gz`, `zip`, directory source

Archive validation must reject:

- absolute paths
- `../` traversal
- symlinks in v1
- device files
- duplicate normalized paths
- undeclared files

---

## 27. Normalized package tree hash

Each package must have a normalized tree hash.

Hash input: relative path, file kind, file mode class, content hash

Rules:

- paths use `/`
- paths are UTF-8
- paths are relative
- paths must not contain `..`
- file order is lexicographic
- symlinks forbidden in v1
- device files forbidden
- only files listed in prayspec included

Pseudo-algorithm:

```
entries = sorted(package_files_by_relative_path)
for each entry:
  append entry.kind
  append "\0"
  append entry.mode_class
  append "\0"
  append entry.path
  append "\0"
  append sha256(entry.bytes)
  append "\n"
tree_hash = sha256(all_appended_bytes)
```

Prayfile.lock records this hash.

---

## 28. Sources

Supported source kinds: registry, static index, git, local path, tarball, OCI artifact, pray SSH

Examples:

```
source "default", "https://agents.example.com"
source "sample", "git+ssh://git@example.com/agent-context/index.git"
source "team", "pray+ssh://pray@prayers.internal"
source "local", path: "../agent-packages"
```

Direct package sources:

```
agent "sample/base", git: "git+ssh://git@example.com/base.git", tag: "v1.4.3"
agent "local/base", path: "../base"
agent "public/base", tarball: "https://example.com/base-1.4.3.praypkg"
agent "public/base", oci: "registry.example.com/agents/base:1.4.3"
```

---

## 29. Static registry protocol

The registry may be a static file tree.

Recommended layout:

```
/v1/index.json
/v1/packages/sample/base.json
/v1/packages/sample/webapp.json
/v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg
```

index.json:

```json
{
  "spec": "prayfile-distribution-1",
  "packages": [
    "sample/base",
    "sample/webapp"
  ]
}
```

Package metadata:

```json
{
  "name": "sample/base",
  "versions": [
    {
      "version": "1.4.3",
      "artifact": "v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg",
      "artifact_hash": "sha256:...",
      "tree_hash": "sha256:...",
      "yanked": false,
      "targets": ["tool_a", "tool_b", "generic"],
      "exports": ["working-agreements", "testing-basics"],
      "derived": {
        "languages": ["markdown"],
        "encodings": ["utf-8"],
        "origins": ["git+ssh://git@example.com/base.git"],
        "summary": "Shared operational guidance for agent use",
        "categories": ["policy", "workflow"],
        "topics": ["testing", "review", "migrations"],
        "file_count": 12,
        "character_count": 18420,
        "token_count": 4120,
        "possible_effects": ["reduce drift", "standardize review output"],
        "possible_side_effects": ["narrower phrasing", "more explicit workflow bias"],
        "embeddings": [
          {
            "model": "local-or-cloud-derived",
            "scope": "package"
          }
        ]
      },
      "confessions": {
        "published_by": "example-maintainer",
        "collected_by": ["sample/base", "prayers.kisko.dev"],
        "received": 18
      }
    }
  ]
}
```

No server API is required for v1. Static hosting must be enough.

### 29.1 Peer-to-peer distribution transport

The specification should also allow a peer-to-peer transport layer for package discovery and artifact seeding.

A conforming implementation may use torrent-style swarms for content distribution and a collective DHT for discovery, inspired by BitTorrent, Freenet, and GNUnet.

Peer-to-peer transport must preserve the same package identity, artifact hash verification, signature checks, yanking semantics, and provenance guarantees as static registry hosting.

P2P transport is optional. A conforming implementation must still work with local, private, and static registry sources without it.

### 29.2 Server-to-server federation

The specification should allow distribution points to form federated networks through explicit server-to-server (S2S) synchronization.

A conforming implementation may support a federation protocol inspired by FIDONet, NNTP, and ActivityPub where:

- Servers establish explicit peer relationships through configuration
- Servers sync package metadata, derived metadata, confessions, and artifacts from trusted peers
- Sync operates on a pull, push, or bidirectional model
- Each server validates packages before accepting them
- Consistency is eventual through periodic synchronization
- Provenance is tracked (origin server, sync path, timestamps)

Federation protocol requirements:

- Discovery endpoint at `/.well-known/pray-federation.json` exposing server capabilities and sync URLs
- Index sync endpoint returning changed packages since a timestamp
- Package metadata sync endpoint with federation-specific fields (origin, publisher, signature, derived metadata, confessions)
- Standard artifact URLs for package file retrieval
- Hash verification and signature validation before acceptance
- Conflict detection for same version with different hashes

Trust levels:

- `full`: Accept metadata, derived metadata, confessions, and artifacts; mirror packages locally
- `metadata_only`: Accept metadata, derived metadata, and confessions but fetch artifacts from origin
- `disabled`: Peer listed but sync paused

Sync directions:

- `pull`: Server fetches updates from peer
- `push`: Server sends updates to peer
- `bidirectional`: Both pull and push

Servers may optionally publish their known peer list, trusted publishers, and confession relay peers to enable discovery of the federation topology.

Federation is optional. A conforming implementation must work without federation support.

### 29.3 Derived metadata and confessions

Distribution points may compute and publish derived metadata for each package version. Derived metadata is an annotation layer, not package identity. It does not change the artifact hash, tree hash, or version identity.

Derived metadata may be computed locally, through cloud inference, or by combining both. Implementations may use language detection, encoding detection, summary generation, topic extraction, embedding generation, and similar analysis tools.

Derived metadata may include:

- detected languages
- detected encodings
- source origins and provenance notes
- summary
- categories
- topics mentioned
- file count
- character count
- token count
- possible effects
- possible side effects
- embeddings

A package may consist only of minimal editable text files intended for alteration. The distribution point may enrich that package with derived metadata without requiring the package itself to carry those annotations.

Confessions are signed usage feedback records. A confession may be produced by a publisher, a distribution point, or a client that has received a package. Confessions may be collected, mirrored, and aggregated by publishers and trusted servers.

Federated servers may share known peer and server lists, along with confessions they are authorized to relay. Publishers may use confessions to collect usage feedback across direct publication and server-to-server synchronization.

Confessions do not alter package identity. They are feedback data attached to a package version, tree hash, or artifact hash.

### 29.4 Zero-trust verification and engine-agnostic annotations

Pray assumes zero trust. Any client, server, publisher, or federation peer may provide incorrect, partial, stale, or malicious data. All metadata, summaries, scores, confessions, and derived annotations are claims unless independently verified or explicitly accepted under local policy.

Package authenticity and injection safety must be verified separately:

- package bytes are verified with artifact hashes and signatures
- package trees are verified with normalized tree hashes
- final injected bytes are verified with exact render digests or equivalent deterministic byte checks
- render plans are verified with canonical metadata about selected exports, exclusions, ordering, normalization, and target policy

Derived metadata may be used for verification, but only as evidence. It can help prove what should be injected, what was excluded, and which inputs were used. It does not become truth simply because it is published by a server.

Any participant may generate annotations using any method, including manual review, hardcoded logic, deterministic heuristics, local inference, cloud inference, or generative models. Implementations must record annotation provenance when they rely on such output, including the producer, method, policy or model version, and the input hash or equivalent binding used to generate the claim.

If conflicting claims are received, local policy decides which claims to trust for discovery or display. Verification of the final injected bytes remains mandatory.

Clients remain unaware of federation. A client queries a single distribution point, which may serve from local mirror, proxy to a peer, or return metadata with origin URLs.

### 29.5 SSH-native distribution transport

A conforming implementation may expose a distribution point over SSH without HTTP. The client opens an SSH session; the server runs `pray serve --stdio` (or an equivalent subsystem entrypoint) and exchanges the same logical registry operations as Section 29 and Section 29.2 through a framed RPC protocol on stdin and stdout.

This transport is optional. A conforming implementation must still support static hosting, HTTP `pray serve`, and the other source kinds in Section 28 without SSH.

#### URL scheme

Pray SSH sources use the `pray+ssh://` scheme:

```
source "team", "pray+ssh://pray@prayers.internal"
source "team", "pray+ssh://pray@prayers.internal:2222"
```

Form:

```
pray+ssh://[<user>@]<host>[:<port>][/<path>]
```

- `user` defaults to implementation policy or the current SSH username
- `port` defaults to `22`
- `path` is an optional hint; the server root is normally fixed by server configuration (`--root`)

Lockfile records:

```toml
[[source]]
name = "team"
kind = "pray_ssh"
url = "pray+ssh://pray@prayers.internal"
```

#### Deployment

A typical private host uses OpenSSH with a forced command or subsystem:

```sshconfig
Subsystem pray /usr/bin/pray serve --stdio --root /var/lib/pray
```

The server stores the same static layout as Section 29 (`v1/index.json`, `v1/packages/...`, `v1/artifacts/...`). No HTTP listener is required.

#### Wire protocol

Spec identifier: `pray-ssh-rpc-v1`

Framing:

```text
frame := u32_be(byte_length) utf8_json
```

Each SSH session carries one or more request/response frame pairs on the server process stdin and stdout.

Request envelope:

```json
{
  "spec": "pray-ssh-rpc-v1",
  "id": "<correlation-id>",
  "method": "<method>",
  "params": {}
}
```

Response envelope:

```json
{
  "spec": "pray-ssh-rpc-v1",
  "id": "<correlation-id>",
  "status": 200,
  "content_type": "application/json",
  "body": {}
}
```

Binary payloads use `content_type: "application/octet-stream"` and `body_encoding: "base64"` on the response, or base64 in `params.body` for uploads.

Conforming implementations must accept frames of at least 16 MiB.

#### RPC methods

RPC methods mirror the reference HTTP distribution API. Params replace path segments and query parameters.

Required methods:

| Method | HTTP equivalent | Params |
|--------|-----------------|--------|
| `federation.discovery` | `GET /.well-known/pray-federation.json` | none |
| `sync.index` | `GET /v1/sync/index` | `since` optional, integer |
| `sync.package` | `GET /v1/sync/package/{name}` | `name` string |
| `sync.push` | `POST /v1/sync/push` | `metadata` package metadata object |
| `artifact.get` | `GET` static artifact path | `path` relative path under server root |
| `artifact.put` | `PUT /v1/artifacts/...` | `path`, `body` base64 |

Optional methods:

| Method | HTTP equivalent |
|--------|-----------------|
| `confession.submit` | `POST /v1/confessions` |
| `auth.register` | `POST /v1/auth/register` |
| `auth.verify` | `POST /v1/auth/verify` |
| `auth.session` | `POST /v1/auth/session` |
| `auth.passkeys.challenge` | `POST /v1/auth/passkeys/challenge` |
| `auth.passkeys.login` | `POST /v1/auth/passkeys/login` |
| `auth.passkeys.enroll` | `POST /v1/auth/passkeys/enroll` |
| `auth.ssh_keys.challenge` | `POST /v1/auth/ssh-keys/challenge` |
| `auth.ssh_keys.login` | `POST /v1/auth/ssh-keys/login` |
| `auth.ssh_keys.enroll` | `POST /v1/auth/ssh-keys/enroll` |

JSON shapes for `federation.discovery`, `sync.index`, `sync.package`, `sync.push`, artifacts, confessions, and auth match the HTTP API and federation types in Section 29.2. HTML index and package pages are not exposed over SSH-RPC.

#### Authentication

SSH-native mode uses SSH for transport authentication:

- host identity via `known_hosts` or equivalent host key pinning (`allowed_host_keys` in client `trust.toml`, optional `host_key_fingerprint` in `Prayfile.lock`)
- user identity via SSH public key fingerprints (`signer_fingerprint` in package metadata, `allowed_publishers` in client trust policy)

The server maps SSH public key fingerprints to publisher identities for push authorization (`v1/ssh_publishers.json`). The reference CLI reads `PRAY_SSH_USER_FINGERPRINT`, `SSH_USER_FINGERPRINT`, or `PRAY_SSH_PUBLISHER` on the server during push. Clients record `signer` (human label) and `signer_fingerprint` (canonical signing identity) in registry metadata; package signatures use the fingerprint when present.

HTTP-style `auth.*` RPC methods are optional and intended for hybrid hosts. SSH-only servers may reject them.

Package hashes, tree hashes, signatures, and render digests are still verified on the client. SSH establishes who connected and encrypts the channel; it does not replace package signature verification.

#### Federation

SSH may be used as a federation transport between peers:

```toml
[[federation.peers]]
name = "team-vps"
transport = "ssh"
url = "pray+ssh://pray@prayers.internal"
trust = "full"
direction = "bidirectional"
```

The logical federation protocol in Section 29.2 is unchanged; only the wire transport differs from HTTP.

### 29.6 Client git source trust policy

A conforming client implementation may enforce optional trust policy for remote `git` sources before resolving packages from a cloned distribution repository.

Policy file location (reference CLI): `~/.pray/trust.toml`. Override with `PRAY_HOME` or `PRAY_USER_HOME` per implementation.

```toml
[default]
allow = true
require_signed_commit = false
allowed_signing_keys = []

[[rules]]
match_prefix = "https://github.com/example/"
require_signed_commit = true
allowed_signing_keys = ["SHA256:ABCDEF..."]
```

Longest `match_prefix` wins; otherwise `[default]` applies.

When policy exists, the client may:

- deny sources with `allow = false`
- require `git verify-commit HEAD` when `require_signed_commit = true`
- restrict signers to `allowed_signing_keys` when that list is non-empty
- prompt for consent when HEAD has no verified-good signature and the signer is not already trusted

SSH-signed commits should use per-source `allowedSignersFile` values scoped to the client's own git subprocesses (`$PRAY_HOME/trust/allowed_signers/`), without modifying the user's global git configuration.

For `pray+ssh` sources, trust rules may also set `allowed_host_keys` (server host key fingerprints) and `allowed_publishers` (SSH user key fingerprints allowed to publish). Package metadata should record `signer_fingerprint` separately from the human-readable `signer` label; signatures use the fingerprint when present.

Reference CLI commands: `pray trust list|show|add-key|remove-key|set-signed|set-allow|import-repo|import-registry|check`. `pray trust import-registry` reads `v1/ssh_publishers.json` from a distribution point and records publisher fingerprints in `allowed_publishers` for the matching rule; for `pray+ssh` sources it also records the server host key in `allowed_host_keys` unless `--no-host-key` is passed. `pray trust check` compares trusted keys against a compromised-key feed (HTTP URL, local file, or git repository).

Global flags: `--trust` imports signer keys after interactive consent; `--global --trust` imports into `[default]`. `PRAY_TRUST_ASSUME_YES=1` auto-consents in non-interactive environments. `--rm` uses an ephemeral `PRAY_HOME` but still copies persistent trust policy into it.

---

## 30. Registry metadata fields

A registry package version should expose:

name, version, summary, description, artifact location, artifact hash, tree hash, yanked flag, license, homepage, source code URI, changelog URI, targets, exports, dependencies, published_at optional, signature optional, render_digest optional, annotation_provenance optional

To reduce churn and privacy leakage, project lockfiles should not copy unnecessary registry metadata.

---

## 31. Lockfile

Prayfile.lock is machine-authored.

Recommended format: TOML.

Reasons: readable, stable, small diffs, easy to parse, good for sorted package tables

Users should not edit Prayfile.lock by hand.

### 31.1 Canonical verification records

Prayfile.lock may include canonical verification records that bind claims to package, render, or confession identities. Verification records are machine-authored and should be stable across implementations.

Recommended format: TOML tables.

Required fields:

| Field | Meaning |
|-------|---------|
| `kind` | Verification subject type: `package`, `render_plan`, `render_output`, or `confession` |
| `subject` | Stable subject reference such as package name and version, managed span ID, confession ID, or artifact reference |
| `subject_hash` | Expected hash for the subject being verified |
| `verifier` | Identity of the client or server that performed the verification |
| `method` | Verification method such as `hash`, `signature`, `manual`, `heuristic`, `local_model`, `cloud_model`, or `rule` |
| `policy` | Policy or trust rule reference used during verification |
| `input_hash` | Hash of the inputs used to produce the claim |
| `observed_hash` | Hash actually observed during verification |
| `observed_at` | Verification timestamp |
| provenance | Origin, source, or federation path for the claim |
| `signature` | Optional signature over the canonical record |

Render-output records should bind the final injected bytes. Render-plan records should also record selected exports, exclusions, ordering, normalization, and target policy in their provenance or detail fields. Confession records should bind the confession body to the sender, package reference, and replay-prevention data.

---

## 32. Lockfile example

```toml
prayfile_lock = "1"
spec = "0.1"
generated_by = "pray 0.1.0"
manifest_hash = "sha256:..."

[[source]]
name = "default"
kind = "registry"
url = "https://agents.example.com"

[[source]]
name = "sample"
kind = "git"
url = "git+ssh://git@example.com/agent-context/index.git"

[[package]]
name = "sample/base"
version = "1.4.3"
source = "sample"
tree_hash = "sha256:..."
artifact_hash = "sha256:..."
artifact = "v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg"
exports = [
  "working-agreements",
  "testing-basics",
  "security-basics",
]

[[package]]
name = "sample/webapp"
version = "2.1.5"
source = "sample"
tree_hash = "sha256:..."
artifact_hash = "sha256:..."
artifact = "v1/artifacts/sample/webapp/2.1.5/sample-webapp-2.1.5.praypkg"
dependencies = [
  "sample/base",
]
exports = [
  "webapp-review",
  "data-layer",
  "testing",
  "live-pages",
]

[[target]]
name = "tool_a"
outputs = [
  "INSTRUCTIONS.md",
  ".tool-a/",
]

[[managed_span]]
id = "p7f3k9m2"
target = "INSTRUCTIONS.md"
open_line = 14
close_line = 20
ideal_checksum = "sha256:abc123..."
package = "sample/base"
export = "testing-basics"
source_checksum = "sha256:def456..."
silenced = false

[[managed_span]]
id = "q8g4h1j6"
target = "INSTRUCTIONS.md"
open_line = 24
close_line = 30
ideal_checksum = "sha256:789abc..."
package = "sample/webapp"
export = "webapp-review"
source_checksum = "sha256:012def..."
silenced = false

[[verification_record]]
kind = "package"
subject = "sample/webapp@2.1.5"
subject_hash = "sha256:..."
verifier = "prayers.kisko.dev"
method = "signature"
policy = "registry-default"
input_hash = "sha256:..."
observed_hash = "sha256:..."
observed_at = "2026-06-29T14:07:56Z"
provenance = "registry"
signature = "ed25519:..."

[[verification_record]]
kind = "render_plan"
subject = "INSTRUCTIONS.md#p7f3k9m2"
subject_hash = "sha256:..."
verifier = "pray 0.1.0"
method = "rule"
policy = "render-managed"
input_hash = "sha256:..."
observed_hash = "sha256:..."
observed_at = "2026-06-29T14:07:56Z"
provenance = "sample/base -> INSTRUCTIONS.md; exports=testing-basics; exclusions=[]"

[[verification_record]]
kind = "render_output"
subject = "INSTRUCTIONS.md#p7f3k9m2"
subject_hash = "sha256:..."
verifier = "pray 0.1.0"
method = "hash"
policy = "render-managed"
input_hash = "sha256:..."
observed_hash = "sha256:..."
observed_at = "2026-06-29T14:07:56Z"
provenance = "final injected bytes"

[[verification_record]]
kind = "confession"
subject = "sample/webapp@2.1.5"
subject_hash = "sha256:..."
verifier = "example-maintainer"
method = "signature"
policy = "confession-default"
input_hash = "sha256:..."
observed_hash = "sha256:..."
observed_at = "2026-06-29T14:07:56Z"
provenance = "publisher"
signature = "ed25519:..."

[[target]]
name = "tool_b"
outputs = [
  "NOTES.md",
  ".tool-b/",
]
```

---

## 32.1 Managed span records

Each managed span (a **prayer** between pray markers) must have a lockfile record.

Required fields:

| Field | Meaning |
|-------|---------|
| `id` | Opaque pray marker ID |
| `target` | Target file path |
| `open_line` | Line number of opening marker |
| `close_line` | Line number of closing marker |
| `ideal_checksum` | Semantic hash of managed body between markers |
| provenance | Package, export, source fragment checksum, silenced flag |

`ideal_checksum` is computed from the managed body only:

* exclude opening and closing pray marker comment lines
* normalize line endings according to target policy
* apply the same semantic hashing rules as `README.md` (pray comments ignored for semantic hash)

`open_line` and `close_line` are 1-based line numbers in the target file after materialization.

Managed span records are updated by `pray apply`, `pray install`, and other materialization commands that explicitly refresh render state. They are not updated by read-only commands.

---

## 32.2 Verify and drift contract

### `pray verify`

Read-only. Compare on-disk target files to managed span records.

For each lockfile managed span:

1. locate the marker pair by `id` in `target`
2. fail if either marker is missing (**removed prayer**)
3. compute semantic checksum of the managed body
4. compare body checksum to `ideal_checksum` (**custom implementation** when different)
5. compare current marker line numbers to `open_line` / `close_line` (**position drift** when checksum matches but lines differ)

`pray verify` reports mismatches. It must not modify `Prayfile.lock` or target files.

Also checks lockfile integrity, package checksums, signatures, cache validity, confession references, and any recorded render digests or annotation provenance.

### `pray apply`

Materializes planned changes, then **refreshes** managed span records:

* rewrite target files when needed
* recompute `ideal_checksum` for each managed span
* recompute `open_line` and `close_line`
* add, update, or remove managed span records

### `pray drift`

Superset of verify. Reports:

| Drift kind | Meaning |
|------------|---------|
| `custom_implementation` | Marker pair exists, but body checksum ≠ `ideal_checksum` |
| `removed_prayer` | Lockfile record exists, marker pair missing from target |
| `position_drift` | Body checksum matches `ideal_checksum`, but marker lines moved |
| `renderer_drift` | On-disk file matches lock, but fresh render from current inputs would change ideals |
| `orphan_marker` | Marker pair in target file has no lockfile managed span record |

`pray drift` does not refresh the lockfile.

---

---

## 33. Lockfile churn rules

To reduce git churn, lockfiles must avoid:

timestamps, absolute paths, local usernames, hostnames except declared sources, cache paths, random IDs, fetch duration, OS-specific path separators, generated file content duplication, machine-specific tool discovery

Stable ordering:

- sources sorted by name
- packages sorted by name/source/version
- targets sorted by name
- arrays sorted unless order is semantic

The lockfile should record: manifest hash, resolved package versions, source identity, artifact hashes, tree hashes, selected exports, dependency graph, and **managed span records** (ideal checksums and marker line positions per prayer).

Per-target `render_hash` may summarize an entire output file. Managed span records are the authoritative per-prayer contract for verify and drift.

It should not duplicate full generated file content. Strict audit mode may optionally record per-file hashes in addition to managed span records.

---

## 34. Manifest hash

`manifest_hash` is a normalized hash of Prayfile.

Normalization process:

1. parse DSL
2. convert to canonical manifest model
3. sort unordered fields
4. preserve semantically meaningful order
5. serialize canonical model
6. hash serialized bytes

Whitespace-only changes should not change `manifest_hash`.

Comment-only changes should not change `manifest_hash`.

---

## 35. Resolver behavior

**Resolver input:** Prayfile, existing Prayfile.lock if present, available sources, target list from manifest, package metadata, cache

**Resolver output:** resolved package graph, selected versions, selected exports, source identities, artifact hashes, tree hashes, target render plan, Prayfile.lock

Resolution rules:

1. Read manifest.
2. Validate syntax.
3. Load existing lockfile if present.
4. Prefer locked versions when they satisfy manifest constraints.
5. Resolve unlocked or changed packages.
6. Resolve transitive dependencies.
7. Reject incompatible versions.
8. Reject missing exports.
9. Reject incompatible targets unless optional.
10. Fetch package artifacts.
11. Verify artifact hash.
12. Verify normalized tree hash.
13. Write lockfile only if resolution changed.

---

## 36. Install behavior

### pray install

Default behavior:

- if lockfile exists and satisfies manifest, use it
- if lockfile missing, resolve and create it
- if manifest changed, minimally re-resolve only necessary packages
- fetch packages
- verify packages
- render target files

### pray install --locked

- require existing Prayfile.lock
- fail if lockfile needs update
- fetch and verify packages
- render only from locked state

### pray install --frozen

- same as `--locked`
- fail if generated files are stale
- fail if verify checks fail
- intended for CI

### pray install --offline

- use cache or vendor directory only
- no network access
- fail if packages unavailable locally

---

## 37. Update behavior

```
pray update
pray update sample/webapp
```

Updates all packages within manifest constraints, or selected package and only dependencies required by that update.

Default update should minimize churn.

Update summary should show: package name, old version, new version, source, exports affected, targets affected, rendered files affected, warnings

Major updates should require explicit intent:

```
pray update sample/webapp --major
```

---

## 38. Remove behavior

```
pray remove sample/webapp
```

Expected behavior:

- remove package declaration from Prayfile
- re-resolve dependency graph
- update Prayfile.lock
- remove generated sections/files no longer needed
- preserve local files
- show diff

---

## 39. Render behavior

**Render input:** Prayfile.lock, resolved package contents, local files, target adapters, render policy

**Render output:** INSTRUCTIONS.md, TOOL_B.md, skill directories, command directories, rule files, target-specific files

Render must be deterministic. Same inputs must produce byte-identical outputs.

---

## 40. Generated file header

Rendered target files may include the ignore marker near the beginning of the file:

```md
<!-- pray:0 ignore-comments -->
```

This marker declares that `pray` comments are render markers and should not be interpreted as instruction content.

The marker is advisory for inference behaviour and binding for Prayfile tooling.

Generated files should not include: timestamps, hostnames, absolute paths, random IDs, or full package graphs unless requested.

---

## 41. Pray markers

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

Each marker ID maps to a managed span record in `Prayfile.lock` storing the ideal checksum and opening/closing line positions for that prayer.

Rendered output cites.
Lockfile explains.
Cache preserves.

Prayfile tooling must ignore pray comments when computing semantic content hashes.

Prayfile tooling may also compute exact file hashes that include marker bytes.

Therefore, implementations may track both:

```text
semantic hash  = rendered content without pray markers
file hash      = exact target file bytes including pray markers
```

Benefits: smaller diffs, easier verify checks, easier partial regeneration, easier human review, drift detection without duplicating provenance in target files

---

## 42. Churn-minimal rendering

When enabled:

```manifest
render churn: :minimal
```

The renderer should:

- keep package order stable
- keep section headings stable
- avoid timestamp changes
- avoid rewrapping paragraphs
- avoid generated package tables unless requested
- avoid large root context
- prefer separate skill files
- preserve local text
- normalize line endings only when configured
- avoid blank-line noise

Preferred output model: small root files, separate skills, separate templates, stable pray markers

Avoid: giant concatenated instruction files, giant duplicated target files, generated walls of generic advice

---

## 43. Local files

Local files are human-owned.

```
local ".agents/project.md", position: :after
```

Rules:

- store human-owned project context under `.agents/` (for example `.agents/project.md`), not under alternate trees such as `agent/local/`
- never overwritten by pray on disk
- content is re-embedded into rendered root files on each render run
- agents edit local source files, not the embed copy inside INSTRUCTIONS.md
- missing local files are errors unless optional
- paths must stay inside repository unless explicitly allowed
- not package dependencies
- not version locked
- not copied into lockfile

Optional local file:

```
local ".agents/private.md", optional: true
```

Local file hashes may be stored in `.pray/state.json`, not Prayfile.lock.

---

## 44. Render modes

| Mode | Behavior |
|------|----------|
| managed | Generated files are owned by pray. Manual edits are overwritten after warning or error depending on policy. Recommended for repositories. |
| check | No files are written. Used by `pray render --check`. |
| local | Generated files are placed into local tool directories. Useful for personal context. |
| vendor | Packages are copied into `.pray/vendor`. Useful for offline and archival workflows. |

---

## 45. Conflict policy

Default:

```manifest
render conflict: :fail
```

Supported policies: `fail`, `warn`, `append`, `last_wins`, `target_specific`

V1 should implement: `fail`, `warn`

Postpone clever merging.

Conflicts include: same generated path with incompatible content, same section ID with different source, duplicate skill name, unsupported target requirement, contradictory target rules, missing export, local file tries to write into managed generated path, package adapter collision

---

## 46. Target adapters

Adapters define how exports become files.

Example `adapters/tool_a.toml`:

```toml
[target]
name = "tool_a"

[root]
file = "INSTRUCTIONS.md"
supports_sections = true
supports_skills = true

[skills]
directory = ".agents/skills"
skill_file = "SKILL.md"

[render]
root_strategy = "summary-plus-sections"
```

Example `adapters/tool_b.toml`:

```toml
[target]
name = "tool_b"

[root]
file = "TOOL_B.md"
supports_sections = true
supports_skills = true

[skills]
directory = ".tool-b/skills"
skill_file = "SKILL.md"
```

Built-in adapters should exist for common targets. Packages may also provide adapters. Project Prayfile may override output paths.

---

## 47. Root context strategy

Root files should stay small.

Recommended root file shape:

```markdown
<!--

Edit `Prayfile`, not this file.
-->
# Agent context
## Additional instructions
...
## Shared instructions
...
## Available capabilities
- code-review
- schema-migration
```

Do not dump every capability body into root files if target supports capabilities.

---

## 48. Example generated instruction file

```markdown
<!-- pray:0 ignore-comments -->

# Input context

Do not edit managed blocks in `AGENTS.md` or skills under `.agents/`.
To change shared guidance, update `Prayfile` and run `pray`.

<!-- pray:l3m8n2p4 -->

## Additional instructions
This repository uses a web stack, test framework, UI library, and relational database.

<!-- pray:l3m8n2p4 -->

## Shared instructions

<!-- pray:p7f3k9m2 -->

### Testing basics
Prefer focused tests near the changed code before broad suites.

<!-- pray:p7f3k9m2 -->

<!-- pray:q8g4h1j6 -->

### Web application review
Check migrations, callbacks, authorization boundaries, background jobs, and data consistency.

<!-- pray:q8g4h1j6 -->

## Available skills
- code-review
- schema-migration
```

---

## 49. Example generated capability

Path: `generated/capabilities/code-review.md`

```markdown
# Code review
## Purpose
Review application changes for correctness, maintainability, and safety.
## When to use
Use when a task changes application code, data models, migrations, jobs, services, or tests.
## Process
1. Inspect changed files.
2. Identify behavior changes.
3. Check tests.
4. Check data and authorization boundaries.
5. Report risks before proposing broad rewrites.
```

---

## 50. CLI command set

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

Additional useful commands:

```
pray remove
pray tree
pray list
pray outdated
pray explain
pray manifest
```

---

## 51. CLI command semantics

| Command | Behavior |
|---------|----------|
| init | Creates minimal Prayfile. Optional: `pray init --targets tool_a,tool_b` |
| add | Adds a package declaration. |
| remove | Removes a package declaration and re-renders. |
| install | Installs according to Prayfile and Prayfile.lock. |
| update | Updates resolved versions. |
| plan | Computes changes to lockfile, cache, and rendered target files. |
| apply | Materializes planned changes and refreshes managed span ideal checksums and line positions. |
| render | Regenerates or verifies target files. Does not replace apply for lock refresh unless documented. |
| format | Normalizes pray markers in target files. |
| verify | Read-only check: managed span checksums, line positions, package integrity, cache, signatures. |
| drift | Reports custom implementation, removed prayers, position drift, renderer drift, and orphan markers. |
| package | Builds `.praypkg`. |
| publish | Packages, signs, and uploads to a distribution point. |
| confess | Signs and submits acceptance or rejection feedback. |
| serve | Runs a local or self-hosted distribution point. |
| vendor | Copies packages into `.pray/vendor`. |
| clean | Removes cache and other local ephemeral state. |
| tree | Shows dependency graph. |

---

## 52. Verify checks

`pray verify` is read-only. It must not modify `Prayfile.lock` or target files.

### Managed span checks

For each `[[managed_span]]` record:

- opening and closing markers with `id` exist in `target`
- managed body semantic checksum equals `ideal_checksum`
- current `open_line` and `close_line` equal lockfile line positions
- report **removed prayer** when lock record exists but marker pair is absent
- report **custom implementation** when markers exist but body checksum ≠ `ideal_checksum`
- report **position drift** when body checksum matches `ideal_checksum` but line positions differ

### Repository checks

`pray verify` should also detect:

- missing Prayfile.lock
- lockfile incompatible with manifest
- package hash mismatch
- missing package source
- missing local files
- orphan pray marker pairs with no lockfile managed span record
- manual edits outside allowed marker regions in managed root files
- duplicate skill names
- unsupported target
- unresolved package source
- path traversal attempt
- vendored package mismatch

Strict mode: `pray verify --strict` turns warnings into errors.

---

## 53. Drift behavior

`pray drift` includes all `pray verify` managed span checks and adds renderer comparison.

Required drift kinds:

| Kind | Detection |
|------|-----------|
| `custom_implementation` | Marker pair present; body checksum ≠ `ideal_checksum` |
| `removed_prayer` | Lock record present; marker pair absent |
| `position_drift` | Body checksum = `ideal_checksum`; marker lines moved |
| `renderer_drift` | On-disk state matches lock; fresh render would change ideals or spans |
| `orphan_marker` | Marker pair present; no lock managed span record |

Required report sections: Lockfile changes, Package changes, Managed span changes, Rendered file changes, Removed prayers, Orphan markers, Warnings

Semantic diff: `pray drift --semantic`

Example output:

```
managed_span q8g4h1j6 INSTRUCTIONS.md
  kind: custom_implementation
  ideal_checksum: sha256:789abc...
  actual_checksum: sha256:111222...
managed_span p7f3k9m2 INSTRUCTIONS.md
  kind: removed_prayer
  expected lines: 14-20
renderer_drift
  sample/webapp 2.1.4 -> 2.1.5 would change 2 managed spans
```

`pray drift` must not refresh the lockfile. Run `pray apply` to accept intentional materialization and refresh managed span records.

---

## 54. CI workflow

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

## 55. Cache layout

Default cache location: `$PRAY_HOME/cache`

If `PRAY_HOME` is unset:

| Platform | Path |
|----------|------|
| Unix-like | `~/.cache/pray` |
| Alternate desktop layout A | `~/Library/Caches/pray` |
| Alternate desktop layout B | `%LOCALAPPDATA%\pray\cache` |

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

---

## 56. State file

`.pray/state.json` is local and ignored.

May contain: last render hashes, manual edit detection data, cache hints, local file hashes, tool discovery result

It must not be required for reproducible install. Deleting it must be safe.

---

## 57. Vendor mode

Vendor mode copies package contents into `.pray/vendor/`.

Used for: offline work, archival, regulated environments, private distribution without registry availability

Vendor mode must preserve package tree hashes. Vendor directory can be committed.

---

## 58. Security model

V1 baseline:

- never execute package code
- reject path traversal
- reject absolute archive paths
- reject symlinks in packages
- verify artifact hashes
- verify tree hashes
- support optional signatures
- support offline mode
- support private sources
- support vendoring
- avoid secrets in lockfile
- avoid environment capture
- avoid timestamps and machine-specific data
- make generated files visibly generated
- make updates explicit

Agent packages affect automated behavior. They must be treated as supply-chain inputs.

---

## 59. Signatures

Optional v1 support:

- package artifact hash signed by publisher key
- registry metadata hash signed by registry key
- lockfile records signature identity

Example lockfile fields:

```
signature = "signed:..."
signer = "sample-agent-packages-2026"
```

Signature support is optional in v1, but the data model should leave space for it.

---

## 60. Yanked packages

Registries may mark versions as yanked.

Rules:

- new resolution should not select yanked versions by default
- existing lockfile may continue using yanked version with warning
- strict mode may fail on yanked version
- update should move away from yanked version when possible

---

## 61. Platform behavior

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

## 62. Tool discovery

Implementations may detect installed tools, but discovery must not affect lockfile resolution unless explicitly requested.

Good:

```
pray verify
# warns: tool_a target configured but tool A not found
```

Bad:

```
pray install
# silently changes lockfile because tool B is installed locally
```

Resolution must be manifest-driven, not machine-driven.

---

## 63. Formatting policy

Formatting is not the product.

Still, a formatter may exist: `pray format`

Rules:

- preserve comments where practical
- do not reorder packages unless requested
- use stable indentation
- prefer one package declaration per dependency
- avoid rewriting whole Prayfile for add/remove
- append new packages at logical location
- make lockfile canonical

Less churn is more important than stylistic perfection.

---

## 64. Global config

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

## 65. Environment variables

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

## 66. Error style

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

Suggested exit codes:

| Code | Meaning |
|------|---------|
| 0 | success |
| 1 | general error |
| 2 | parse error |
| 3 | resolution error |
| 4 | integrity error |
| 5 | render/check failed |
| 6 | verify failed |
| 7 | network/fetch error |
| 8 | unsupported feature |

---

## 67. Semantic versioning for packages

Agent packages should use semantic versioning.

**Patch** — Small clarification that should not materially change behavior.

Examples: typo fix, wording clarification, broken link fix, small example correction

**Minor** — Backward-compatible new guidance.

Examples: new export, new optional skill, new target adapter, expanded checklist

**Major** — Behavior-changing instruction update.

Examples: changed testing policy, changed security policy, removed export, renamed skill, changed default target behavior, new strict rule

Agent context affects behavior, so package authors should not hide meaningful changes in patch versions.

---

## 68. Changelog

Recommended package file: `CHANGELOG.md`

Package manifest may declare:

```manifest
spec.changelog_uri = "https://example.com/sample/webapp/CHANGELOG.md"
```

`pray update` should display changelog entries when available.

---

## 69. Package author best practices

Package authors should:

- keep exports small
- make exports topic-specific
- avoid giant root context
- prefer skills for large procedures
- prefer templates for reusable output forms
- avoid vague rules
- avoid generic AI advice
- avoid private data
- avoid secrets
- avoid conflicting instructions
- document when to use each export
- document target compatibility
- keep changelogs
- use semantic versioning

Good package split: `sample/base`, `sample/security`, `sample/webapp`, `sample/frontend`, `sample/testing`, `sample/accessibility`, `sample/deployment`

Bad package split: `sample/everything`

---

## 70. Repository author best practices

Repository authors should:

- keep project-local instructions short
- put reusable instructions into packages
- commit Prayfile.lock
- review generated diffs
- use frozen CI mode
- update packages intentionally
- avoid editing generated files
- split local context by topic
- remove dead instructions
- prefer small root files
- prefer skills for longer material

---

## 71. Private context

Private context should be handled through: private registry, local path packages, ignored local files, optional local files, vendor mode

Do not put secrets, credentials, personal facts, private business data, or private customer data into public packages.

Do not put secrets into Prayfile.lock.

---

## 72. Conformance levels

| Level | Capability | Required commands |
|-------|------------|-------------------|
| 0: Parser | Can parse Prayfile and *.prayspec | `pray manifest` |
| 1: Installer | Can resolve local/path/tarball packages and produce lockfile | `pray install`, `pray verify` |
| 2: Renderer | Can render at least one target | `pray render`, `pray render --check` |
| 3: Package manager | Supports distribution point, Git, update, drift, verify | `pray update`, `pray drift`, `pray verify` |
| 4: Publisher | Can pack and publish packages to static registry | `pray package`, `pray publish` |

The reference implementation should target Level 3 first.

---

## 73. Test fixtures

The open specification should include conformance fixtures:

```
fixtures/
  parser/
  prayspec/
  resolver/
  lockfile/
  render/
  verify/
  registry/
  packages/
```

Each fixture should contain: input Prayfile, input prayspec files, package sources, expected canonical manifest, expected Prayfile.lock, expected rendered files, expected diagnostics

This allows independent implementations.

---

## 74. Minimal v1 implementation scope

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

## 75. Suggested reference implementation architecture

Crate/module split:

```
pray-cli
pray-core
pray-parser
pray-resolve
pray-lock
pray-render
pray-package
pray-distribution
pray-verify
```

Internal layers:

```
CLI (pray)
  parses commands
Core
  orchestrates resolve and render
Parser
  Prayfile DSL
  prayspec DSL
Model
  canonical manifest
  package spec
  lockfile model
  render plan
Resolve
  semver resolution
  sources
  dependency graph
  lockfile writes
Render
  fetch registry / git / path / tarball
  verify artifact hash
  verify tree hash
  optional signatures
  target adapters
  generated files
  pray markers
  churn-minimal writing
Doctor
  drift checks
  stale files
  conflict checks
```

Suggested dependency categories:

```
CLI argument parsing
serialization and schema codecs
lockfile and adapter formats
version constraint parsing
cryptographic hashing
UTF-8 path handling
archive read/write
compression codecs
filesystem traversal
text diffing
structured error reporting
```

Parser choice: Start with a strict hand-written parser or parser combinator libraries. Do not embed a host language. Do not use eval.

---

## 76. Open specification repository layout

Suggested repository:

```
prayfile-spec/
  README.md
  SPEC.md
  CHANGELOG.md
  LICENSE
  schema/
    manifest.schema.json
    lockfile.schema.json
    registry.schema.json
    package.schema.json
  examples/
    minimal/
    webapp/
    multi-target/
    private-registry/
  fixtures/
    parser/
    prayspec/
    resolver/
    render/
    verify/
  rfcs/
    0001-prayfile.md
    0002-prayspec.md
    0003-registry.md
```

Suggested implementation repo:

```
pray/
  crates/
    pray-cli/
    pray-core/
    pray-parser/
    pray-resolve/
    pray-render/
    pray-package/
  tests/
  README.md
```

---

## 77. First milestone

Milestone 1 should produce:

- parse Prayfile
- parse prayspec
- load local path package
- resolve exact version
- write Prayfile.lock
- render INSTRUCTIONS.md
- verify check generated file

No registry yet.

Example demo:

```
pray init --targets tool_a
pray add local/base --path ../agent-packages/base
pray install
cat INSTRUCTIONS.md
pray verify
```

---

## 78. Second milestone

Milestone 2:

- package build
- .praypkg archive
- tree hash
- tarball source
- static registry source
- install --locked
- install --frozen
- render --check
- diff

---

## 79. Third milestone

Milestone 3:

- git source
- update command
- semantic diff
- tool_b target
- tool-specific rendering
- vendor mode
- offline mode
- publish static registry
- conformance fixtures

---

## 80. Ownership and generated-output contract

The hardest part of Prayfile is keeping **managed** rendered output stable and read-only while **local** additions remain editable and safe from overwrite.

The model is not one shared rendered file everyone edits. It is three zones with different owners.

### Three zones

| Zone | Source | Who edits | What pray does |
|------|--------|-----------|----------------|
| **Recipe** | Prayfile, packages, Prayfile.lock | Humans via `pray add`, `pray remove`, `pray update` | resolves and locks |
| **`.agents`** | `.agents/**` (human-owned; `.agents/skills/**` is package-managed) | Humans and applications | reads on render; **never writes** |
| **Managed** | `AGENTS.md`, generated target files, package-owned rules | Nobody directly | fully regenerates from lock + recipe + `.agents` inputs |

Package exports live only in the managed zone. They are pinned by recipe and hash. Applications consume them; they do not rewrite them.

Human-owned files under `.agents/` (outside package-managed `.agents/skills/**`) are not locked, not hashed in `Prayfile.lock`, and are re-embedded into rendered output on every `pray render` when listed in `Prayfile`.

### Golden rules

1. Applications must **not** edit managed files or managed blocks.
2. Applications **may** edit human-owned files under `.agents/` when project-specific input must change.
3. Humans change shared packages by editing **Prayfile** and running **pray**, not by patching rendered target files.
4. Render reconstructs managed output from inputs. There is no three-way merge in v1.

### Render composition

Root files are assembled in a fixed order:

```
preamble              # short contract (generated)
embedded inputs       # files listed in Prayfile under `.agents/`
managed blocks        # one block per package export
index                 # names only; bodies live elsewhere
```

Managed blocks use opaque pray markers from section 41:

```md
<!-- pray:p7f3k9m2 -->

...rendered content...

<!-- pray:p7f3k9m2 -->
```

On render, pray replaces each managed block from locked package content and re-embeds listed `.agents` files into their spans. Anything outside allowed marker regions is a **verify error**.

### Target preamble

Every generated root file may start with a short, user-facing contract. It must not mention implementation details.

Recommended shape:

```markdown
<!-- pray:0 ignore-comments -->

# Input context

Do not edit managed blocks in `AGENTS.md` or skills under `.agents/`.
To change shared guidance, update `Prayfile` and run `pray`.
```

The ignore marker is for tooling. The visible lines are for the application.

### Managed output ownership

Managed output installs under the target directory for the current project.

Each managed directory or file must carry origin metadata, either in front matter or a small `.pray-origin.toml`:

```toml
package = "sample/webapp"
export = "code-review"
version = "2.1.5"
tree_hash = "sha256:..."
```

Optional human-owned files under `.agents/` are not origin-tagged as packages. Name collisions between human-owned and managed content are **conflicts** unless policy says otherwise.

Applications must not edit managed directories. They may edit other files under `.agents/`.

### Idempotency

**Definition:** same inputs must yield the same managed bytes.

Inputs to render:

- Prayfile.lock
- resolved package trees (verified by tree hash)
- `.agents/**` contents listed in `Prayfile`
- render policy from Prayfile
- target adapter

Guarantees:

| Command | Idempotent when |
|---------|-----------------|
| `pray install` | lock and inputs unchanged → no writes (optional optimization) |
| `pray render` | always reconstructs managed zones from inputs |
| `pray install --frozen` | fails instead of mutating when output would change |
| render phase (internal) | same lock + same local files + same packages → byte-identical managed output |

Local file edits change only local embeds and the root file hash. They do not require resolve unless Prayfile changed.

Package updates change only managed blocks owned by affected packages.

### Update behavior

```
pray update sample/webapp
```

1. resolve selects new version within constraints and updates Prayfile.lock.
2. render replaces every managed block mapped to `sample/webapp` in `Prayfile.lock`.
3. render replaces managed directories whose origin package is `sample/webapp`.
4. Embedded `.agents` files are re-read but not modified on disk.
5. `pray drift` shows recipe, lock, managed-block, and render changes.

Pray markers make updates surgical in diffs even though render is logically full reconstruction.

### Remove behavior

```
pray remove sample/webapp
```

1. Remove declaration from Prayfile.
2. resolve recomputes lock without that package.
3. render deletes all managed blocks mapped to `sample/webapp`.
4. render deletes managed directories tagged with that package origin.
5. Human-owned `.agents/**` files are preserved.
6. Orphan pray markers after remove are **verify errors**.

### Verify enforcement

`pray verify` is read-only. `pray drift` extends it with renderer comparison.

Must detect:

- **custom implementation** — managed body checksum ≠ lockfile `ideal_checksum`
- **removed prayer** — lockfile managed span exists; marker pair missing
- **position drift** — body checksum matches `ideal_checksum`; marker lines moved
- **orphan marker** — marker pair exists; no lock managed span record
- manual edits inside managed directories
- content outside any allowed marker region in managed root files
- stale render relative to lock and local inputs
- missing local files referenced by Prayfile
- duplicate managed names across local and managed paths
- invalid, nested, or unmatched pray markers

Strict mode turns all of these into errors. CI uses `pray install --frozen`, `pray verify --strict`, and `pray drift`.

To refresh ideal checksums and line positions after intentional changes, run `pray apply`.

### Why this works

Applications are untrusted editors. Treat managed rendered output like compiled output:

```
Prayfile + packages  →  resolve  →  lock
lock + local + packages  →  render  →  rendered targets
```

If an application rewrites a managed block, the next `pray render` or CI frozen check fails. The fix is not merge logic. The fix is regenerate and move custom text into `.agents/` or update `Prayfile`.

That keeps shared packages stable, local overrides editable, and the system idempotent.

---

## 81. Final principle

This system should not try to make inference smarter by adding more magic.

It should make inference input boring enough to trust.

The valuable thing is not another prompt folder.

The valuable thing is:

```
declared input
locked input
verified input
rendered input with compact citations
recoverable source fragments
explicit silence
```

That is the whole point of Prayfile, Prayfile.lock, *.prayspec, and pray.
