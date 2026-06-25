# Prayfile Open Specification

**Status:** Draft v0.1  
**Primary file names:** Prayfile, Prayfile.lock, *.prayspec, *.praypkg  
**Reference CLI name:** pray  
**Project name:** pray  
**Reference implementation target:** systems language  
**Specification goal:** language-independent, platform-independent, implementation-independent

---

## 1. Summary

Prayfile is an open specification for reproducible inference input composition.

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

Modern inference engines rely on surrounding input files such as `AGENTS.md`, `CLAUDE.md`, instruction libraries, prompt templates, review checklists, memories, formatting rules, and workflow notes. These files are often distributed manually through copy-paste. Prayfile lets projects declare shared input dependencies, resolve them deterministically, lock exact versions and content hashes, preserve original source fragments, and render tool-specific outputs with compact provenance markers.

**FAQ:**

| Question | Answer |
|----------|--------|
| Is this a prompt framework? | No. The durable problem is packaging and distributing the material placed before inference—not prompt design itself. |
| What is input drift? | The gradual divergence of instructions, policies, templates, memories, formatting rules, and workflow assumptions between projects. |
| Why now? | Cross-tool support for `AGENTS.md`, `CLAUDE.md`, and similar files removed a major adoption blocker. Copy-paste still does not scale. |
| Is the spec final? | No. Draft v0.1 is an experiment. Terminology, formats, and behaviour may evolve as the model is validated. |
| Implementation status? | Spec-first. Reference CLI design lives in this document and `README.md`. |

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
| Auditable traces | Every managed rendered span carries a compact pray marker. `Prayfile.lock` records exact resolved state, source checksums, silenced fragments, and provenance metadata. |
| Temporal clarity | Lockfile and drift semantics show what changed between resolves. Version control carries when. Pray markers enable surgical rollback, blame, and review without rereading entire target files. |
| Measurable effects | Effects are measured at the dependency boundary first: manifest → lock → rendered bytes → reviewable diff. Inference behaviour remains human-validated; the specification does not score model quality. |
| Security | Input packages are supply-chain inputs: static declarations only, hash-verified, path-safe, explicitly updated, optionally signed. Audit trails align with integrity—implementations can prove what was installed, from where, and at which version. |

These values inform lockfile fields, pray markers, `pray drift` output, `pray verify` checks, and the security model in later sections.

### Experiment intent

Packaging shapes, tool conventions, and workflow surfaces for inference input will keep changing drastically—skills today, something else tomorrow. This specification is an experiment in *seeing* that motion, not in freezing one workflow bet.

To observe change, you need indicators. Prayfile defines them as contracts: pinned lock state, pray markers, explicit diffs, integrity checks, and signed feedback. The core values above are those indicators made normative—so teams can measure what altered, when, and from where while the surrounding ecosystem shifts.

---

## 3. Problem

Inference-oriented development now commonly uses files and folders such as:

- AGENTS.md
- CLAUDE.md
- `.github/copilot-instructions.md`
- `.agents/`
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
- support for public/private/local distribution
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

### RubyGems alignment

Prayfile is Bundler-shaped for resolve and lock, with an additional render phase. RubyGems and Bundler are the closest reference ecosystem; the core values in section 2 extend their indicator model to inference-input dependencies.

| Prayfile | RubyGems / Bundler |
|-----------|-------------------|
| Prayfile | Gemfile |
| Prayfile.lock | Gemfile.lock |
| *.prayspec | *.gemspec |
| *.praypkg | `.gem` |
| resolve (lock) | resolver / `bundle lock` |
| render (fetch + materialize) | no direct equivalent — gems install as code trees, not merged context files |
| pray verify | `bundle check` and sanity checks |
| pray drift | lockfile diff plus rendered-output diff |

| Core value | RubyGems / Bundler | Prayfile extension |
|------------|-------------------|---------------------|
| Auditable traces | lockfile pins; package name and version | compact pray markers inside rendered target files |
| Temporal clarity | lockfile history; yanked gems; explicit `bundle update` | `pray drift` across lock and render; marker-level blame and rollback |
| Measurable effects | manifest → lock → install; behavior validated by tests | manifest → lock → rendered bytes → diff; inference behaviour stays human-validated |
| Security | checksums; yanked gems; optional signing; vendoring | same supply-chain baseline; packages are static declarations only — no host-language execution |

Prayfile does not replace RubyGems. It applies reproducibility and audit patterns proven necessary for code dependencies to inference-input dependencies: lock what resolved, render what landed, cite managed spans compactly in target files.

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

---

## 9. Repository layout

Recommended project layout:

```
Prayfile
Prayfile.lock
AGENTS.md
CLAUDE.md
.github/copilot-instructions.md

.pray/cache/                # ignored by default
.pray/vendor/               # optional, committed only in hermetic/offline mode

agent/local/                # optional human-owned overrides
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
- commit rendered target files such as `AGENTS.md` and `CLAUDE.md` when tools require repository-local files
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
local "agent/local/project.md"
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
      "path": "agent/local/project.md",
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
local "agent/local/project.md"
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
local "agent/local/project.md", position: :after
local "agent/local/testing.md", position: :after
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
source "local", path: "../agent-packages"
```

Source names must be unique.

Supported source kinds: `registry`, `static_index`, `git`, `path`, `tarball`, `oci`

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

Groups dependencies.

```manifest
group :webapp do
  agent "sample/webapp", "~> 2.1"
  agent "sample/ui-kit", "~> 1.0"
end
```

Groups are organizational unless explicitly connected to targets or features.

### local

Includes human-owned local project context.

```
local "agent/local/project.md"
local "agent/local/security.md", position: :after
local "agent/local/private.md", optional: true
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
| fragment | Markdown/text fragment rendered into root files |
| skill | Directory containing SKILL.md |
| template | Reusable text artifact |
| command | Tool-specific or generic command template |
| rule | Tool-specific rule file |
| asset | Static file used by skill/template |
| bundle | Named collection of other exports |

---

## 24. Skills

A skill export is a directory containing `SKILL.md`.

Optional: `assets/`, `templates/`, `examples/`

Recommended SKILL.md structure:

```
# Skill name
## Purpose
## When to use
## Inputs
## Process
## Output
```

Skill directories must be copied deterministically.

Two packages must not install the same skill path unless conflict policy allows it.

---

## 25. Package payload rules

V1 packages are data packages.

Allowed package contents: Markdown, TOML, JSON, YAML, plain text, templates, declared assets, images/diagrams if useful for skills, scripts as inert assets only

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

Supported source kinds: registry, static index, git, local path, tarball, OCI artifact

Examples:

```
source "default", "https://agents.example.com"
source "sample", "git+ssh://git@example.com/agent-context/index.git"
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
      "exports": ["working-agreements", "testing-basics"]
    }
  ]
}
```

No server API is required for v1. Static hosting must be enough.

---

## 30. Registry metadata fields

A registry package version should expose:

name, version, summary, description, artifact location, artifact hash, tree hash, yanked flag, license, homepage, source code URI, changelog URI, targets, exports, dependencies, published_at optional, signature optional

To reduce churn and privacy leakage, project lockfiles should not copy unnecessary registry metadata.

---

## 31. Lockfile

Prayfile.lock is machine-authored.

Recommended format: TOML.

Reasons: readable, stable, small diffs, easy to parse, good for sorted package tables

Users should not edit Prayfile.lock by hand.

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
  ".agents/skills",
]
render_hash = "sha256:..."

[[target]]
name = "tool_b"
outputs = [
  "TOOL_B.md",
  ".tool-b/skills",
]
render_hash = "sha256:..."
```

---

## 33. Lockfile churn rules

To reduce git churn, lockfiles must avoid:

timestamps, absolute paths, local usernames, hostnames except declared sources, cache paths, random IDs, fetch duration, OS-specific path separators, generated file content duplication, machine-specific tool discovery

Stable ordering:

- sources sorted by name
- packages sorted by name/source/version
- targets sorted by name
- arrays sorted unless order is semantic

The lockfile should record: manifest hash, resolved package versions, source identity, artifact hashes, tree hashes, selected exports, dependency graph, target render hashes

It should not record every generated file hash by default. Strict audit mode may optionally record per-file hashes.

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
local "agent/local/project.md", position: :after
```

Rules:

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
local "agent/local/private.md", optional: true
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

Edit Prayfile or agent/local/*.md, not this file.
-->
# Agent context
## Project-local instructions
...
## Shared instructions
...
## Available skills
- code-review
- schema-migration
```

Do not dump every skill body into root files if target supports skills.

---

## 48. Example generated AGENTS.md

```markdown
<!-- pray:0 ignore-comments -->

# Agent context

Do not edit managed blocks or managed skills.
Add or change project-specific instructions in `agent/local/` only.
To change shared guidance, ask a human to update `Prayfile` and run `pray`.

<!-- pray:l3m8n2p4 -->

## Project-local instructions
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

## 49. Example generated skill

Path: `.agents/skills/code-review/SKILL.md`

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
| apply | Materializes planned changes. |
| render | Regenerates or verifies target files. |
| format | Normalizes pray markers in target files. |
| verify | Validates lockfile integrity, checksums, signatures, cache, and target consistency. |
| drift | Checks whether managed blocks differ from lockfile and renderer output. |
| package | Builds `.praypkg`. |
| publish | Packages, signs, and uploads to a distribution point. |
| confess | Signs and submits acceptance or rejection feedback. |
| serve | Runs a local or self-hosted distribution point. |
| vendor | Copies packages into `.pray/vendor`. |
| clean | Removes cache and other local ephemeral state. |
| tree | Shows dependency graph. |

---

## 52. Verify checks

`pray verify` should detect:

- missing Prayfile.lock
- lockfile incompatible with manifest
- package hash mismatch
- missing package source
- missing local files
- stale generated files
- manual edits inside managed pray-marker blocks
- manual edits inside managed skill directories
- content outside allowed marker regions in managed root files
- duplicate sections
- duplicate skill names
- unsupported target
- root file too large
- context bloat warning
- unresolved package source
- path traversal attempt
- generated files not listed in render plan
- vendored package mismatch
- line ending inconsistency
- unknown DSL statement

Strict mode: `pray verify --strict` turns warnings into errors.

---

## 53. Diff behavior

`pray drift` required sections: Lockfile changes, Package changes, Rendered file changes, Removed files, Added files, Warnings

Semantic diff: `pray drift --semantic`

Example output:

```
sample/webapp 2.1.4 -> 2.1.5
Changed exports:
  testing
    + added focused system spec guidance
    - removed obsolete browser driver note
Rendered files:
  INSTRUCTIONS.md
  .agents/skills/code-review/SKILL.md
```

---

## 54. CI workflow

Recommended CI:

```
pray install --frozen
pray verify --strict
pray render --check
```

CI must fail when:

- lockfile is missing
- lockfile needs update
- package hash mismatch exists
- generated files are stale
- generated files were manually edited
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
```

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
- skills rendering
- vendor mode
- offline mode
- publish static registry
- conformance fixtures

---

## 80. Ownership and agent contract

The hardest part of Prayfile is keeping **managed** rendered output stable and read-only for agents while **local** additions remain editable and safe from overwrite.

The model is not “one big `AGENTS.md` everyone edits.” It is three zones with different owners.

### Three zones

| Zone | Source | Who edits | What pray does |
|------|--------|-----------|----------------|
| **Recipe** | Prayfile, packages, Prayfile.lock | Humans via `pray add`, `pray remove`, `pray update` | resolves and locks |
| **Local** | `agent/local/**` | Humans and agents | reads on render; **never writes** |
| **Managed** | Generated target files, package skills, package rules | Nobody directly | fully regenerates from lock + recipe + local inputs |

Package exports and skills live only in the managed zone. They are pinned by recipe and hash. Agents consume them; they do not rewrite them.

Local overrides live only under `agent/local/`. They are not locked, not hashed in `Prayfile.lock`, and are re-embedded into rendered output on every `pray render`.

### Golden rules

1. Agents must **not** edit managed files, managed blocks, or managed skill directories.
2. Agents **may** edit `agent/local/**` when project-specific input must change.
3. Humans change shared packages by editing **Prayfile** and running **pray**, not by patching rendered target files.
4. Render reconstructs managed output from inputs. There is no three-way merge in v1.

### Render composition

Root files are assembled in a fixed order:

```
preamble              # short contract (generated)
local embeds          # copied from agent/local/*.md
managed blocks        # one block per package export
skills index          # names only; bodies live in skill dirs
```

Managed blocks use opaque pray markers from section 41:

```md
<!-- pray:p7f3k9m2 -->

...rendered content...

<!-- pray:p7f3k9m2 -->
```

On render, pray replaces each managed block from locked package content and re-embeds local files into their managed spans. Anything outside allowed marker regions is a **verify error**.

### Target preamble

Every generated root file may start with a short, user-facing contract. It must not mention implementation details.

Recommended shape:

```markdown
<!-- pray:0 ignore-comments -->

# Agent context

Do not edit managed blocks or managed skills.
Add or change project-specific instructions in `agent/local/` only.
To change shared guidance, ask a human to update `Prayfile` and run `pray`.
```

The ignore marker is for tooling. The visible lines are for the agent.

### Skills ownership

Package skills install under the target skills directory, for example `.agents/skills/`.

Each managed skill directory must carry origin metadata, either in `SKILL.md` front matter or a small `.pray-origin.toml`:

```toml
package = "sample/webapp"
skill = "code-review"
version = "2.1.5"
tree_hash = "sha256:..."
```

Optional local skills live under `agent/local/skills/` and copy to `.agents/skills/local/<name>/`. They are not origin-tagged as packages. Name collisions between local and managed skills are **conflicts** unless policy says otherwise.

Agents must not edit managed skill directories. They may edit `agent/local/skills/`.

### Idempotency

**Definition:** same inputs must yield the same managed bytes.

Inputs to render:

- Prayfile.lock
- resolved package trees (verified by tree hash)
- `agent/local/**` contents
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

Package updates change only blocks and skills owned by affected packages.

### Update behavior

```
pray update sample/webapp
```

1. resolve selects new version within constraints and updates Prayfile.lock.
2. render replaces every managed block mapped to `sample/webapp` in `Prayfile.lock`.
3. render replaces skill directories whose origin package is `sample/webapp`.
4. Local embeds and `agent/local/**` are re-read but not modified on disk.
5. `pray drift` shows recipe, lock, managed-block, skill, and render changes.

Pray markers make updates surgical in diffs even though render is logically full reconstruction.

### Remove behavior

```
pray remove sample/webapp
```

1. Remove declaration from Prayfile.
2. resolve recomputes lock without that package.
3. render deletes all managed blocks mapped to `sample/webapp`.
4. render deletes managed skills tagged with that package origin.
5. `agent/local/**` is preserved.
6. Orphan pray markers after remove are **verify errors**.

### Verify enforcement

`pray verify` is the guardrail when agents or humans edit the wrong zone.

Must detect:

- manual edits inside managed pray-marker blocks
- manual edits inside managed skill directories
- content outside any allowed marker region in managed root files
- stale render relative to lock and local inputs
- missing local files referenced by Prayfile
- duplicate skill names across local and managed paths
- invalid, nested, or unmatched pray markers

Strict mode turns all of these into errors. CI uses `pray install --frozen` and `pray verify --strict`.

### Why this works

Agents are untrusted editors. Treat managed rendered output like compiled output:

```
Prayfile + packages  →  resolve  →  lock
lock + local + packages  →  render  →  AGENTS.md + skills
```

If an agent rewrites a managed block in `AGENTS.md`, the next `pray render` or CI frozen check fails. The fix is not merge logic. The fix is regenerate and move custom text into `agent/local/`.

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
