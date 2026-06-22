# Agentfile Open Specification

**Status:** Draft v0.1  
**Primary file names:** Agentfile, Agentfile.lock, *.agentspec, *.agentpkg  
**Reference CLI name:** pata  
**Mix phase name:** seko  
**Brew phase name:** uuta  
**Project name:** pata  
**Reference implementation target:** systems language  
**Specification goal:** language-independent, platform-independent, implementation-independent

---

## 1. Summary

Agentfile is an open, lockfile-based dependency specification for reusable AI-agent context.

It allows repositories to declare shared agent instructions, templates, workflows, review rules, and tool-specific context packages in a reproducible way. Targets may render skills or other tool-specific artifacts; the specification is workflow-neutral.

The core model is:

| Concept | Role |
|---------|------|
| Agentfile | human-authored dependency manifest |
| Agentfile.lock | machine-authored exact resolved state |
| *.agentspec | package definition file |
| *.agentpkg | built package archive |
| pata | reference CLI and project name |
| seko | mix phase — resolve dependencies and merge exports |
| uuta | brew phase — fetch, verify, and render context |

Agentfile is conceptually similar to dependency manifest.

Agentfile.lock is conceptually similar to dependency lockfile.

*.agentspec is conceptually similar to *.packagespec.

But unlike legacy package registries, the specification must not require host-language execution. All files must be parseable as static declarations by any implementation in any language.

The goal is not to create a magic agent. The goal is to distribute, lock, verify, and render agent context cleanly—with provenance for every provisioned alteration.

---

## 2. Core positioning

**One-sentence definition:**

Agentfile is an open, lockfile-based package specification for reusable AI-agent context, allowing repositories to declare shared instructions once, resolve them reproducibly, render small tool-specific context files with reviewable diffs, and mark every provisioned section with its source.

**Short pitch:**

Agent context is becoming a dependency, but today it is distributed by copy-paste. There is no bundler for these text files. Agentfile introduces a lockfile-resolver-like model for AGENTS.md, `.agents/`, INSTRUCTIONS.md, TOOL_B.md, templates, and tool-specific instruction files. Agentfile declares desired context packages, Agentfile.lock records the exact resolved versions and hashes, and pata renders small deterministic outputs for different agent tools—with section markers and origin tags so every provisioned alteration has a visible source.

**FAQ:**

| Question | Answer |
|----------|--------|
| Is this an "agentic skills" platform? | No. Skills may be one artifact type a target renders. The durable problem is distributing, locking, and rendering text context with reviewable diffs and provenance—not betting on a single workflow shape. |
| What is "context alteration"? | Automated tools assemble working context from shared libraries and local additions. Agentfile defines contracts for that assembly: resolve, lock, render, and mark provisioned sections with package origin. |
| Why now? | Cross-tool support for AGENTS.md and `.agents/` removed a major adoption blocker. Copy-paste still does not scale. |
| Is the spec final? | No. Draft v0.1 is an experiment in a fast-moving space. The model may be reworked as its indicators prove—or fail—at making change visible. |
| Implementation status? | Spec-first. Reference CLI design lives in this document. |

**Implementation mantra:**

```
Parse data.
Resolve deterministically.
Lock exactly.
Render minimally.
Never execute package code.
Never hide updates.
Keep diffs small.
```

### Core values

Context alteration is permanent—it shapes how automated tools inspect, edit, test, and reason about code. Agentfile treats observability and trust as first-class requirements, not optional polish.

| Value | What it means |
|-------|---------------|
| Auditable traces | Every provisioned alteration carries package origin. Agentfile.lock records exact resolved state. Render output is visibly generated and bounded by stable section markers and origin tags. |
| Temporal clarity | Lockfile and diff semantics show what changed between resolves. Version control carries when. Section markers enable surgical rollback, blame, and review without rereading entire context files. |
| Measurable effects | Effects are measured at the dependency boundary first: manifest → lock → rendered bytes → reviewable diff. Behavioral outcomes remain human-validated; the specification does not score agent quality. |
| Security | Context packages are supply-chain inputs: static declarations only, hash-verified, path-safe, explicitly updated. Audit trails align with integrity—implementations can prove what was installed, from where, and at which version. |

These values inform lockfile fields, section markers, `pata diff` output, doctor checks, and the security model in later sections.

### Experiment intent

Packaging shapes, tool conventions, and workflow surfaces for agent context will keep changing drastically—skills today, something else tomorrow. This specification is an experiment in *seeing* that motion, not in freezing one workflow bet.

To observe change, you need indicators. Agentfile defines them as contracts: pinned lock state, provenance markers, explicit diffs, and integrity checks. The core values above are those indicators made normative—so teams can measure what altered, when, and from where while the surrounding ecosystem shifts.

---

## 3. Problem

Agent-oriented development now commonly uses files and folders such as:

- INSTRUCTIONS.md
- TOOL_B.md
- .agents/
- .tool-b/skills/
- .tool-c/rules/
- .tool-d/
- .tool-e/
- prompts/
- review-checklists/
- security-guides/
- testing-guides/
- workflow templates

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
- different output for different agent tools
- accidental private-context leakage
- giant INSTRUCTIONS.md / TOOL_B.md files

Agent context is not just documentation. It affects how automated tools inspect, edit, test, and reason about code.

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
- auditable provenance for every provisioned alteration
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
- a bet that current agent-context conventions will stay unchanged; it assumes they will evolve while context alteration remains

Self-recovery means deterministic reconstruction from Agentfile.lock.

Self-update means explicit `pata update`.

It must not mean hidden mutation.

---

## 6. Naming

Recommended names:

| Concept | Name |
|---------|------|
| Spec / manifest | Agentfile |
| Lockfile | Agentfile.lock |
| Package spec | *.agentspec |
| Package archive | *.agentpkg |
| CLI | pata |
| Mix / resolve phase | seko |
| Brew / render phase | uuta |
| Project / implementation | pata |
| Implementation crate/package | pata or pata-agentfile |
| Registry concept | Agentfile Registry |

Etymology (ASCII-friendly):

**pata** (*pot*, *cauldron*) — the CLI and project name. Everything mixes and brews in the pot.

**seko** (from *sekoitus*, *mixture*; verb *sekoittaa*, to mix) — the mix phase: parse Agentfile, resolve versions, merge exports, write Agentfile.lock.

**uuta** (from *uuttaa*, to brew or extract) — the brew phase: fetch packages, verify hashes, render target files.

Typical pipeline: **seko** then **uuta**. `pata install` runs both.

**seko** and **uuta** are internal phase and module names only. They are not CLI commands or aliases.

**Agentfile** and **Agentfile.lock** stay as the manifest and lockfile names.

Example command usage:

```
pata init
pata add sample/webapp "~> 2.1"
pata install
pata update
pata doctor
pata render
pata pack
pata publish
```

Semantic analogy:

| Agentfile concept | Analogy |
|-------------------|---------|
| Agentfile | recipe |
| Agentfile.lock | exact brew record |
| agent package | ingredient / volume |
| export | portion |
| registry | pantry |
| install | seko + uuta |
| update | seko (re-mix) |
| render | uuta (brew output) |
| doctor | taste test |
| vendor | jar on the shelf |

---

## 7. Ecosystem analogy

| Reference package ecosystem | Agentfile ecosystem |
|----------------|---------------------|
| dependency manifest | Agentfile |
| dependency lockfile | Agentfile.lock |
| *.packagespec | *.agentspec |
| .legacy-archive | .agentpkg |
| resolver install | pata install |
| resolver update | pata update |
| package build | pata pack |
| package publish | pata publish |
| legacy package registry | Agentfile registry / static index |

**Important difference:**

Legacy registries may execute host-language code.  
Agentfile must parse declarations only.

### RubyGems alignment

Agentfile is Bundler-shaped for resolve and lock, with an additional render phase. RubyGems and Bundler are the closest reference ecosystem; the core values in section 2 extend their indicator model to context dependencies.

| Agentfile | RubyGems / Bundler |
|-----------|-------------------|
| Agentfile | Gemfile |
| Agentfile.lock | Gemfile.lock |
| *.agentspec | *.gemspec |
| *.agentpkg | `.gem` |
| seko (resolve + lock) | resolver / `bundle lock` |
| uuta (fetch + render) | no direct equivalent — gems install as code trees, not merged context files |
| pata doctor | `bundle check` and sanity checks |
| pata diff | lockfile diff plus rendered-output diff |

| Core value | RubyGems / Bundler | Agentfile extension |
|------------|-------------------|---------------------|
| Auditable traces | lockfile pins; package name and version | section markers and origin tags inside rendered context files |
| Temporal clarity | lockfile history; yanked gems; explicit `bundle update` | `pata diff` across lock and render; section-level blame and rollback |
| Measurable effects | manifest → lock → install; behavior validated by tests | manifest → lock → rendered bytes → diff; agent behavior stays human-validated |
| Security | checksums; yanked gems; optional signing; vendoring | same supply-chain baseline; packages are static declarations only — no host-language execution |

Agentfile does not replace RubyGems. It applies reproducibility and audit patterns proven necessary for code dependencies to context dependencies: lock what resolved, render what landed, mark every provisioned alteration with its source.

---

## 8. Terminology

**Agentfile** — Human-authored dependency manifest.

**Agentfile.lock** — Machine-authored exact resolved state.

**agentspec** — Package definition file.

**agent package** — Versioned bundle of agent-context content.

**export** — Named unit provided by a package.

Examples: `webapp-review`, `testing-guidance`, `security-basics`, `ui-components`, `incident-template`

**target** — An agent tool or output environment.

Examples: `tool_a`, `tool_b`, `tool_c`, `tool_d`, `tool_e`, `generic`

**adapter** — Mapping from generic package exports into target-specific files.

**render** — Process of creating actual target files from locked package state.

**managed file** — Generated file owned by Pata.

**local file** — Human-owned project file included or appended into rendered output.

**source** — Place where packages are resolved from.

Examples: registry, static index, git, local path, tarball, OCI artifact, file share

**frozen install** — Install mode that refuses to update lockfile or generated files.

---

## 9. Repository layout

Recommended project layout:

```
Agentfile
Agentfile.lock
agent/
  local/
    project.md
    testing.md
    security.md
    skills/              # optional human/agent-owned skills
INSTRUCTIONS.md               # generated if tool A target enabled
TOOL_B.md               # generated if tool B target enabled
.agents/skills/         # generated if target uses skills
.tool-b/skills/         # generated if target uses skills
.tool-c/rules/          # generated if tool C target enabled
.agentfile/
  cache/                # ignored
  state.json            # ignored
  vendor/               # optional, committed only in vendor mode
```

Recommended `.gitignore`:

```
.agentfile/cache/
.agentfile/state.json
```

Depending on repository policy, generated target files may be committed or ignored.

---

## 10. Commit policy

**Recommended default for most repositories:**

- commit Agentfile
- commit Agentfile.lock
- commit generated INSTRUCTIONS.md / TOOL_B.md if tools require repository-local files
- commit generated skills if tools require repository-local skills
- ignore cache
- ignore state

**Recommended for local personal context:**

- commit Agentfile
- optionally commit Agentfile.lock
- ignore generated local tool output
- ignore cache
- ignore state

**Recommended for offline / archival workflows:**

- commit Agentfile
- commit Agentfile.lock
- commit `.agentfile/vendor`
- commit generated files if target tools need them

---

## 11. Agentfile design

Agentfile is a declarative declarative manifest DSL.

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
agentfile "1"
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

Every valid Agentfile compiles to a canonical language-neutral model.

Example:

```json
{
  "agentfile_version": "1",
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

## 13. Minimal Agentfile example

```manifest
agentfile "1"
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

## 14. Larger Agentfile example

```manifest
agentfile "1"
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

## 15. Agentfile statements

### agentfile

Declares spec version.

```
agentfile "1"
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
  sample-webapp.agentspec
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

Required: `*.agentspec`

Optional: `README.md`, `LICENSE`, `CHANGELOG.md`, `exports/`, `skills/`, `templates/`, `adapters/`, `assets/`

---

## 20. agentspec design

`*.agentspec` is the package definition file. It is inspired by legacy `.packagespec`. It is declarative but not executable host language.

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
  spec.agentfile_version = ">= 0.1"
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
    "agentfile.target.tool_a" => "true",
    "agentfile.target.tool_b" => "true"
  }
end
```

---

## 21. agentspec allowed grammar

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
homepage= source_code_uri= changelog_uri= agentfile_version= files=
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

## 22. agentspec canonical model

Every `*.agentspec` compiles to a canonical package model:

```json
{
  "name": "sample/webapp",
  "version": "2.1.5",
  "summary": "web applications, testing, data layer, and live UI agent context",
  "license": "MIT",
  "agentfile_version": ">= 0.1",
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

Packages may contain executable-looking files only as inert assets. Agentfile must not execute them.

---

## 26. Package archive

Built package file: `sample-webapp-2.1.5.agentpkg`

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
- only files listed in agentspec included

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

Agentfile.lock records this hash.

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
agent "public/base", tarball: "https://example.com/base-1.4.3.agentpkg"
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
/v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.agentpkg
```

index.json:

```json
{
  "spec": "agentfile-registry-1",
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
      "artifact": "v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.agentpkg",
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

Agentfile.lock is machine-authored.

Recommended format: TOML.

Reasons: readable, stable, small diffs, easy to parse, good for sorted package tables

Users should not edit Agentfile.lock by hand.

---

## 32. Lockfile example

```toml
agentfile_lock = "1"
spec = "0.1"
generated_by = "pata 0.1.0"
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
artifact = "v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.agentpkg"
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
artifact = "v1/artifacts/sample/webapp/2.1.5/sample-webapp-2.1.5.agentpkg"
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

`manifest_hash` is a normalized hash of Agentfile.

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

**Resolver input:** Agentfile, existing Agentfile.lock if present, available sources, target list from manifest, package metadata, cache

**Resolver output:** resolved package graph, selected versions, selected exports, source identities, artifact hashes, tree hashes, target render plan, Agentfile.lock

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

### pata install

Default behavior:

- if lockfile exists and satisfies manifest, use it
- if lockfile missing, resolve and create it
- if manifest changed, minimally re-resolve only necessary packages
- fetch packages
- verify packages
- render target files

### pata install --locked

- require existing Agentfile.lock
- fail if lockfile needs update
- fetch and verify packages
- render only from locked state

### pata install --frozen

- same as `--locked`
- fail if generated files are stale
- fail if doctor checks fail
- intended for CI

### pata install --offline

- use cache or vendor directory only
- no network access
- fail if packages unavailable locally

---

## 37. Update behavior

```
pata update
pata update sample/webapp
```

Updates all packages within manifest constraints, or selected package and only dependencies required by that update.

Default update should minimize churn.

Update summary should show: package name, old version, new version, source, exports affected, targets affected, rendered files affected, warnings

Major updates should require explicit intent:

```
pata update sample/webapp --major
```

---

## 38. Remove behavior

```
pata remove sample/webapp
```

Expected behavior:

- remove package declaration from Agentfile
- re-resolve dependency graph
- update Agentfile.lock
- remove generated sections/files no longer needed
- preserve local files
- show diff

---

## 39. Render behavior

**Render input:** Agentfile.lock, resolved package contents, local files, target adapters, render policy

**Render output:** INSTRUCTIONS.md, TOOL_B.md, skill directories, command directories, rule files, target-specific files

Render must be deterministic. Same inputs must produce byte-identical outputs.

---

## 40. Generated file header

Generated files should include a compact header:

```html
<!--
Generated by Pata from Agentfile.
Edit Agentfile or agent/local/*.md, not this file.
-->
```

Generated files should not include: timestamps, hostnames, absolute paths, random IDs, full package graph unless requested

---

## 41. Section markers

Generated root files should use stable section markers:

```html
<!-- pata:begin package="sample/base" export="testing-basics" -->
## Testing basics
Run focused tests before broad suites.
<!-- pata:end package="sample/base" export="testing-basics" -->
```

Benefits: smaller diffs, easier doctor checks, easier partial regeneration, easier human review, clear source attribution

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

Preferred output model: small root files, separate skills, separate templates, stable section markers

Avoid: giant concatenated INSTRUCTIONS.md, giant duplicated TOOL_B.md, generated walls of generic advice

---

## 43. Local files

Local files are human-owned.

```
local "agent/local/project.md", position: :after
```

Rules:

- never overwritten by pata on disk
- content is re-embedded into rendered root files on each uuta run
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

Local file hashes may be stored in `.agentfile/state.json`, not Agentfile.lock.

---

## 44. Render modes

| Mode | Behavior |
|------|----------|
| managed | Generated files are owned by Pata. Manual edits are overwritten after warning or error depending on policy. Recommended for repositories. |
| check | No files are written. Used by `pata render --check`. |
| local | Generated files are placed into local tool directories. Useful for personal context. |
| vendor | Packages are copied into `.agentfile/vendor`. Useful for offline and archival workflows. |

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

Built-in adapters should exist for common targets. Packages may also provide adapters. Project Agentfile may override output paths.

---

## 47. Root context strategy

Root files should stay small.

Recommended root file shape:

```markdown
<!--
Generated by Pata from Agentfile.
Edit Agentfile or agent/local/*.md, not this file.
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

## 48. Example generated INSTRUCTIONS.md

```markdown
<!--
Generated context. Do not edit this file.
Project-specific rules: agent/local/
Shared rules: changed via repository recipe, not here.
-->
# Agent context

Do not edit provisioned sections or provisioned skills.
Add or change project-specific instructions in `agent/local/` only.
To change shared guidance, ask a human to update the repository recipe and reinstall.

<!-- pata:begin local path="agent/local/project.md" -->
## Project-local instructions
This repository uses a web stack, test framework, UI library, and relational database.
<!-- pata:end local path="agent/local/project.md" -->

## Shared instructions
<!-- pata:begin package="sample/base" export="testing-basics" -->
### Testing basics
Prefer focused tests near the changed code before broad suites.
<!-- pata:end package="sample/base" export="testing-basics" -->
<!-- pata:begin package="sample/webapp" export="webapp-review" -->
### Web application review
Check migrations, callbacks, authorization boundaries, background jobs, and data consistency.
<!-- pata:end package="sample/webapp" export="webapp-review" -->

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

Minimum v1 commands:

```
pata init
pata add
pata remove
pata install
pata update
pata render
pata diff
pata doctor
pata tree
pata verify
pata pack
pata publish
```

Useful later commands:

```
pata list
pata outdated
pata explain
pata cache clean
pata vendor
pata export
pata normalize
pata format
pata manifest
```

---

## 51. CLI command semantics

| Command | Behavior |
|---------|----------|
| init | Creates minimal Agentfile. Optional: `pata init --targets tool_a,tool_b` |
| add | Adds a package declaration. |
| remove | Removes a package declaration and re-renders. |
| install | Installs according to Agentfile and Agentfile.lock. |
| update | Updates resolved versions. |
| render | Regenerates or verifies target files. |
| diff | Shows what would change. |
| doctor | Validates repository state. |
| verify | Verifies hashes and signatures. |
| tree | Shows dependency graph. |
| pack | Builds .agentpkg. |
| publish | Publishes package to registry or static index. |

---

## 52. Doctor checks

`pata doctor` should detect:

- missing Agentfile.lock
- lockfile incompatible with manifest
- package hash mismatch
- missing package source
- missing local files
- stale generated files
- manual edits inside `pata:begin` / `pata:end` provisioned blocks
- manual edits inside provisioned skill directories
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

Strict mode: `pata doctor --strict` turns warnings into errors.

---

## 53. Diff behavior

`pata diff` required sections: Lockfile changes, Package changes, Rendered file changes, Removed files, Added files, Warnings

Semantic diff: `pata diff --semantic`

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
pata install --frozen
pata doctor --strict
pata render --check
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

Default cache location: `$AGENTFILE_HOME/cache`

If `AGENTFILE_HOME` is unset:

| Platform | Path |
|----------|------|
| Unix-like | `~/.cache/agentfile` |
| Alternate desktop layout A | `~/Library/Caches/agentfile` |
| Alternate desktop layout B | `%LOCALAPPDATA%\agentfile\cache` |

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

`.agentfile/state.json` is local and ignored.

May contain: last render hashes, manual edit detection data, cache hints, local file hashes, tool discovery result

It must not be required for reproducible install. Deleting it must be safe.

---

## 57. Vendor mode

Vendor mode copies package contents into `.agentfile/vendor/`.

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
pata doctor
# warns: tool_a target configured but tool A not found
```

Bad:

```
pata install
# silently changes lockfile because tool B is installed locally
```

Resolution must be manifest-driven, not machine-driven.

---

## 63. Formatting policy

Formatting is not the product.

Still, a formatter may exist: `pata format`

Rules:

- preserve comments where practical
- do not reorder packages unless requested
- use stable indentation
- prefer one package declaration per dependency
- avoid rewriting whole Agentfile for add/remove
- append new packages at logical location
- make lockfile canonical

Less churn is more important than stylistic perfection.

---

## 64. Global config

Optional user config: `~/.config/agentfile/config.toml`

May contain:

```toml
[cache]
directory = "~/.cache/agentfile"

[network]
offline = false

[registry.default]
url = "https://agents.example.com"
```

Global config must not inject packages into a repository. The repository dependency graph must come from Agentfile.

---

## 65. Environment variables

Allowed implementation variables:

```
AGENTFILE_HOME
AGENTFILE_CACHE
AGENTFILE_CONFIG
AGENTFILE_NO_COLOR
AGENTFILE_OFFLINE
```

They may affect local behavior. They must not silently change package resolution.

---

## 66. Error style

Errors must be precise.

Good:

```
Agentfile:14: package "sample/webapp" requests export "testing2", but available exports are: "testing", "data-layer", "webapp-review".
```

Bad:

```
Resolution failed.
```

Error categories: `parse_error`, `manifest_error`, `resolution_error`, `fetch_error`, `integrity_error`, `render_error`, `doctor_error`, `target_error`

Suggested exit codes:

| Code | Meaning |
|------|---------|
| 0 | success |
| 1 | general error |
| 2 | parse error |
| 3 | resolution error |
| 4 | integrity error |
| 5 | render/check failed |
| 6 | doctor failed |
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

`pata update` should display changelog entries when available.

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
- commit Agentfile.lock
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

Do not put secrets into Agentfile.lock.

---

## 72. Conformance levels

| Level | Capability | Required commands |
|-------|------------|-------------------|
| 0: Parser | Can parse Agentfile and *.agentspec | `pata manifest` |
| 1: Installer | Can resolve local/path/tarball packages and produce lockfile | `pata install`, `pata verify` |
| 2: Renderer | Can render at least one target | `pata render`, `pata render --check` |
| 3: Package manager | Supports registry, Git, update, diff, doctor | `pata update`, `pata diff`, `pata doctor` |
| 4: Publisher | Can pack and publish packages to static registry | `pata pack`, `pata publish` |

The reference implementation should target Level 3 first.

---

## 73. Test fixtures

The open specification should include conformance fixtures:

```
fixtures/
  parser/
  agentspec/
  resolver/
  lockfile/
  render/
  doctor/
  registry/
  packages/
```

Each fixture should contain: input Agentfile, input agentspec files, package sources, expected canonical manifest, expected Agentfile.lock, expected rendered files, expected diagnostics

This allows independent implementations.

---

## 74. Minimal v1 implementation scope

Reference implementation should support:

- Agentfile parser
- agentspec parser
- Agentfile.lock reader/writer
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
- section markers
- install
- update
- render
- diff
- doctor
- verify
- pack
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
pata-cli
pata-core
pata-parser
pata-seko
pata-lock
pata-uuta
pata-package
pata-registry
pata-doctor
```

Internal layers:

```
CLI (pata)
  parses commands
Core
  orchestrates seko and uuta
Parser
  Agentfile DSL
  agentspec DSL
Model
  canonical manifest
  package spec
  lockfile model
  render plan
Seko
  semver resolution
  sources
  dependency graph
  lockfile writes
Uuta
  fetch registry / git / path / tarball
  verify artifact hash
  verify tree hash
  optional signatures
  target adapters
  generated files
  section markers
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
agentfile-spec/
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
    agentspec/
    resolver/
    render/
    doctor/
  rfcs/
    0001-agentfile.md
    0002-agentspec.md
    0003-registry.md
```

Suggested implementation repo:

```
pata/
  crates/
    pata-cli/
    pata-core/
    pata-parser/
    pata-seko/
    pata-uuta/
    pata-package/
  tests/
  README.md
```

---

## 77. First milestone

Milestone 1 should produce:

- parse Agentfile
- parse agentspec
- load local path package
- resolve exact version
- write Agentfile.lock
- render INSTRUCTIONS.md
- doctor check generated file

No registry yet.

Example demo:

```
pata init --targets tool_a
pata add local/base --path ../agent-packages/base
pata install
cat INSTRUCTIONS.md
pata doctor
```

---

## 78. Second milestone

Milestone 2:

- package build
- .agentpkg archive
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

The hardest part of Agentfile is keeping **provisioned** context stable and read-only for agents while **local** additions remain editable and safe from overwrite.

The model is not “one big INSTRUCTIONS.md everyone edits.” It is three zones with different owners.

### Three zones

| Zone | Source | Who edits | What pata does |
|------|--------|-----------|----------------|
| **Recipe** | Agentfile, packages, Agentfile.lock | Humans via `pata add`, `pata remove`, `pata update` | **seko** resolves and locks |
| **Local** | `agent/local/**` | Humans and agents | Reads on render; **never writes** |
| **Provisioned** | Generated root files, package skills, package rules | Nobody directly | **uuta** fully regenerates from lock + recipes + local inputs |

**Potions** (package exports and skills) live only in the provisioned zone. They are pinned by recipe and hash. Agents consume them; they do not rebrew them.

**Local mixes** live only under `agent/local/`. They are not locked, not hashed in Agentfile.lock, and are re-embedded into rendered output on every `pata render`.

### Golden rules

1. Agents must **not** edit provisioned files or provisioned sections.
2. Agents **may** edit `agent/local/**` when project-specific context must change.
3. Humans change shared potions by editing **Agentfile** and running **pata**, not by patching INSTRUCTIONS.md.
4. Render reconstructs provisioned output from inputs. There is no three-way merge in v1.

### Render composition

Root files are assembled in a fixed order:

```
agent preamble          # short contract (generated)
local embeds            # copied from agent/local/*.md
provisioned sections    # one block per package export
skills index            # names only; bodies live in skill dirs
```

Local embeds use stable markers:

```html
<!-- pata:begin local path="agent/local/project.md" -->
...content copied from file...
<!-- pata:end local path="agent/local/project.md" -->
```

Provisioned exports keep section markers from section 41:

```html
<!-- pata:begin package="sample/base" export="testing-basics" -->
...
<!-- pata:end -->
```

On render, pata replaces the entire local embed from the current file bytes and replaces each provisioned block from locked package content. Anything outside these markers is a **doctor error**.

### Agent preamble

Every generated root file starts with a short, user-facing contract. It must not mention implementation details.

Recommended shape:

```markdown
<!--
Generated context. Do not edit this file.
Project-specific rules: agent/local/
Shared rules: changed via repository recipe, not here.
-->
# Agent context

Do not edit provisioned sections or provisioned skills.
Add or change project-specific instructions in `agent/local/` only.
To change shared guidance, ask a human to update the repository recipe and reinstall.
```

The HTML comment is for tools. The visible lines are for the agent.

### Skills ownership

Package skills install under the target skills directory, for example `.agents/skills/`.

Each provisioned skill directory must carry origin metadata, either in `SKILL.md` front matter or a small `.pata-origin.toml`:

```toml
package = "sample/webapp"
skill = "code-review"
version = "2.1.5"
tree_hash = "sha256:..."
```

Optional local skills live under `agent/local/skills/` and copy to `.agents/skills/local/<name>/`. They are not origin-tagged as packages. Name collisions between local and provisioned skills are **conflicts** unless policy says otherwise.

Agents must not edit provisioned skill directories. They may edit `agent/local/skills/`.

### Idempotency

**Definition:** same inputs must yield the same provisioned bytes.

Inputs to **uuta**:

- Agentfile.lock
- resolved package trees (verified by tree hash)
- `agent/local/**` contents
- render policy from Agentfile
- target adapter

Guarantees:

| Command | Idempotent when |
|---------|-----------------|
| `pata install` | lock and inputs unchanged → no writes (optional optimization) |
| `pata render` | always reconstructs provisioned zones from inputs |
| `pata install --frozen` | fails instead of mutating when output would change |
| uuta phase (internal) | same lock + same local files + same packages → byte-identical provisioned output |

Local file edits change only local embeds and the root file hash. They do not require **seko** unless Agentfile changed.

Package updates change only blocks and skills owned by affected packages.

### Update behavior

```
pata update sample/webapp
```

1. **seko** selects new version within constraints and updates Agentfile.lock.
2. **uuta** replaces every root section where `package="sample/webapp"`.
3. **uuta** replaces skill directories whose origin package is `sample/webapp`.
4. Local embeds and `agent/local/**` are re-read but not modified on disk.
5. `pata diff` shows recipe, lock, section, skill, and render changes.

Section markers make updates surgical in diffs even though render is logically full reconstruction.

### Remove behavior

```
pata remove sample/webapp
```

1. Remove declaration from Agentfile.
2. **seko** recomputes lock without that package.
3. **uuta** deletes all root sections tagged `package="sample/webapp"`.
4. **uuta** deletes provisioned skills tagged with that package origin.
5. `agent/local/**` is preserved.
6. Orphan markers or origin tags after remove are **doctor errors**.

### Doctor enforcement

`pata doctor` is the guardrail when agents or humans edit the wrong zone.

Must detect:

- manual edits inside `pata:begin package=...` / `pata:end` blocks
- manual edits inside provisioned skill directories
- content outside any allowed marker region in managed root files
- stale render relative to lock and local inputs
- missing local files referenced by Agentfile
- duplicate skill names across local and provisioned paths

Strict mode turns all of these into errors. CI uses `pata install --frozen` and `pata doctor --strict`.

### Why this works

Agents are untrusted editors. Treat provisioned context like compiled output:

```
Agentfile + packages  →  seko  →  lock
lock + local + packages  →  uuta  →  INSTRUCTIONS.md + skills
```

If an agent rewrites a potion in INSTRUCTIONS.md, the next `pata render` or CI frozen check fails. The fix is not merge logic. The fix is regenerate and move custom text into `agent/local/`.

That keeps distributed potions stable, local mixes editable, and the system idempotent.

---

## 81. Final principle

This system should not try to make agents smarter by adding more magic.

It should make agent context boring enough to trust.

The valuable thing is not another prompt folder.

The valuable thing is:

```
declared context
locked context
verified context
minimal rendered context
reviewable context updates
recoverable context state
```

That is the whole point of Agentfile, Agentfile.lock, *.agentspec, and pata.
