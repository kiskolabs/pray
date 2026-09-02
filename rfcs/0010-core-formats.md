# RFC 0010: Core formats

- Feature Name: core-formats
- Type: Standards Track
- Status: Stable
- Describes: 1.8.1
- Created: 2026-08-17
- Author: Andrei Makarov
- Relates: RFC 0020, RFC 0030, RFC 0102, RFC 0011, RFC 0034

## Summary

Prayfile is the human-authored manifest. Meaning is the canonical model after parse. The parser rejects host-language execution. Lockfile rules live in RFC 0020. Package rules live in RFC 0011.

## Motivation

Independent implementations need one meaning for a Prayfile regardless of Ruby-like DSL formatting. Whitespace-only source changes MUST NOT change resolution.

## Guide-level explanation

A project commits Prayfile and usually Prayfile.lock. `pray install` writes the lock and renders destinations. `pray format` rewrites the manifest toward `compose` / `tree` / `pray …, file:`.

Deprecated keywords `target`, `output`, `agent`, and `skills` still parse and warn. Changelog 1.6.0 schedules removal in CLI version 2. RFC 0102 tracks making destination DSL the documented default.

## Reference-level explanation

This section is the product contract for this concern. Where it disagrees with Implementation notes, Implementation notes record what the reference CLI does today. A follow-on RFC records the gap.

### 11. Prayfile design

Prayfile is a declarative manifest DSL.

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
  folder ".agents/skills"
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

#### Editor language mode

`Prayfile` and `Prayfile.lock` are extensionless; editors may mis-detect language. Pin by filename until a `prayfile` grammar exists. Practical defaults: associate `Prayfile` with Ruby (or plaintext) and `Prayfile.lock` with TOML. Disable workspace language detection if auto-detect keeps flipping. Spec does not require a particular editor.

---

### 12. Canonical manifest model

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

### 13. Minimal Prayfile example

```manifest
prayfile "1"
source "default", "https://agents.example.com"
target :tool_a do
  output "INSTRUCTIONS.md"
  folder ".agents/skills"
end
target :tool_b do
  output "TOOL_B.md"
  folder ".tool-b/skills"
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

### 14. Larger Prayfile example

```manifest
prayfile "1"
source "default", "https://agents.example.com"
source "sample", "git+ssh://git@example.com/agent-context/index.git"
target :tool_a do
  output "INSTRUCTIONS.md"
  folder ".agents/skills"
  max_bytes 120_000
end
target :tool_b do
  output "TOOL_B.md"
  folder ".tool-b/skills"
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
  header: true
```

---

### 15. Prayfile statements

#### prayfile

Declares spec version.

```
prayfile "1"
```

Required.

#### source

Declares package source.

```
source "default", "https://agents.example.com"
source "sample", "git+ssh://git@example.com/agent-index.git"
source "team", "pray+ssh://pray@prayers.internal"
source "local", path: "../agent-packages"
```

Source names must be unique.

Supported source kinds: `registry`, `static_index`, `git`, `path`, `tarball`, `oci`, `pray_ssh`

#### target

Optional grouping for legacy Prayfiles and selective apply. Not required when using top-level `compose` / `tree` / `pray …, file:`.

Deprecated in prayfile `"1"`: implementations should warn that `target`, nested `output`, and nested `skills` will be removed in version 2. Prefer `compose` / `tree` / `folder`. Top-level `output "path" do … end` (compose alias) is likewise deprecated in favor of `compose`. Top-level `skills "path" do … end` is likewise deprecated in favor of `tree`.

```manifest
target :tool_a do
  output "INSTRUCTIONS.md"
  folder ".agents/skills"
end
```

Common target fields:

```
output "INSTRUCTIONS.md"
folder ".agents/skills"
commands ".tool-b/commands"
rules ".tool-c/rules"
max_bytes 120_000
```

Unknown target features should warn by default. Strict mode should fail.

#### compose

Builds one text file from ordered local embeds and package fragments.

```manifest
compose "AGENTS.md" do
  pray ".agents/project.md"
  pray "amkisko/working-rules", "~> 2.0"
end
```

Rules:

- Declaration order inside the block is render order.
- Package inputs use fragment exports (see default export resolution).
- Local bare paths embed human-owned files (no pray markers).
- Markers are HTML comments (`<!-- pray:id -->`) only. Compose fails closed unless the destination accepts HTML comments, as specified by RFC 0108.
- Alias: `output "AGENTS.md" do … end` at top level.

#### tree

Provisions package folder exports under a directory root.

```manifest
tree ".agents/skills" do
  pray "amkisko/engineering-audit", "~> 2.0"
end
```

Alias: `folder`. Deprecated alias: `skills` (block form at top level); removed in version 2.

Rules:

- Copies listed folder leaves into the destination directory.
- Leaves undeclared siblings in place.
- Destinations are project-relative. A leading `~` is a manifest error (RFC 0033).

#### pray

Primary input sugar for packages and (inside `compose`) local files.

```manifest
pray "amkisko/working-rules", "~> 2.0"
pray "amkisko/community-security", "~> 1.0", file: "SECURITY.md"
pray ".agents/project.md"
```

Aliases: `use`, `include`. Legacy `agent` / `package` remain valid in prayfile `"1"`. Implementations should warn that `agent` is deprecated and will be removed in version 2; prefer `pray`.

Forms:

- `pray "owner/name", "constraint", …`: Package
- `pray "owner/name", …, file: "path"`: Package file export at path (after `((pray:…))` substitution)
- `pray "relative/or/./path"` bare, inside `compose`: Local file embed
- `pray do … end`: Default symbol map for `((pray:…))` placeholders

`file:` rules:

- Requires a `file`-typed package export (default export resolution applies).
- Writes UTF-8 text after `((pray:…))` substitution (binary non-UTF-8 copies as bytes); no pray markers; no agent header.
- Exclusive ownership of the path. Refuse-clobber, symlink dest reject, lock ledger, and hash-gated prune are RFC 0033.
- Destination strings MUST be project-relative. A leading `~` is a manifest error, not home expansion.
- Mutually exclusive with nesting inside `compose` / `tree`.
- Optional alias: `file "SECURITY.md" do pray "pkg", "~> 1.0" end`.

#### pray symbols (templating)

Project-wide string symbols for package and local content. Declared once; applied to every compose fragment, local embed, and `file:` / tree text export.

```manifest
pray do
  support_email "contact@kiskolabs.com"
  security_email "security@kiskolabs.com"
end
```

Alias: `template do … end`.

Surface sugar (optional, normalizes to the canonical form above):

- `{ … }` blocks instead of `do` / `end` when the statement is a keyword call (`pray{…}`, `compose("AGENTS.md"){…}`, `compose "AGENTS.md"{…}`); assignment map literals such as `spec.exports = { … }` are left alone
- top-level `;` statement separators (including one-liners)
- optional call parentheses after keywords (`compose("AGENTS.md") do`)
- optional call parentheses on symbol assignments (`support_email("…")`)

Still forbidden: interpolation, constants, variables, method chaining, and other executable Ruby.

Placeholder form (strict, no spaces):

```
((pray:<path>))
```

Grammar:

- `Placeholder := "((" "pray" ":" Path "))"`
- `Path := [A-Za-z0-9._/-]+`

Examples: `((pray:support_email))`, `((pray:user.email))`.

Rules:

- Unknown `((pray:…))` symbols fail render.
- Spaced forms such as `(( pray:email ))` are not placeholders.
- Only the `pray` resolver is implemented; other resolvers such as `((env:…))` are reserved for later.
- Verify compares provisioned UTF-8 files to the substituted expected content, not the raw package bytes.

Default export resolution when `export:` / `exports:` omitted:

- inside `compose`: `fragment`
- `file:` keyword: `file`
- inside `tree`: `folder` (deprecated alias `skill`)

Exactly one compatible export is selected; multiple require `export: "name"`; none is a type mismatch. Legacy-only Prayfiles (no `compose` / `tree` / `pray` / `file:`) keep empty exports selecting all package exports.

Package name prefixes are namespaces. When a namespace matches a declared source handle name, `source:` may be omitted. `source:` is also optional when only one source exists.

#### agent

Declares package dependency (legacy primary form; prefer `pray`).

```manifest
agent "sample/webapp", "~> 2.1",
  exports: ["webapp-review", "testing"]
```

Supported options:

```
source: :sample
export: "name"
exports: [...]
targets: [...]
features: [...]
optional: true
file: "SECURITY.md"
git: "..."
tag: "..."
rev: "..."
path: "..."
tarball: "..."
oci: "..."
```

#### group

Groups package declarations for environment-aware rendering.

```manifest
group :development, :test do
  agent "sample/webapp", "~> 2.1"
  agent "sample/ui-kit", "~> 1.0"
end
```

Rules:

- A group block must use `do ... end` and may list multiple environment names separated by commas.
- Only `agent`, `package`, or `pray` / `use` declarations are allowed inside a group block.
- Nested group blocks are rejected.
- Packages outside any group always render.
- When no render environment is selected, only ungrouped packages render.
- When `PRAY_ENV` or `--env` / `--environment` selects a name, ungrouped packages plus packages whose `groups` include that name render.
- Unknown environment names fail with the available group names.
- Group membership is part of the canonical manifest and manifest hash.
- Package resolution and lock entries remain complete for every declared package regardless of the selected environment; only rendered managed spans and provisioned files are filtered.

#### local

Includes human-owned local project context.

```
local ".agents/project.md"
local ".agents/security.md", position: :after
local ".agents/security.md", at: :start
local ".agents/private.md", optional: true
```

Inside `compose`, prefer `pray ".agents/project.md"` so order follows block declaration order.

Supported positions: `before`, `after`, `target_after` (`at:` is an alias for `position:`)

Default: `after`

#### Compatibility

Stay on `prayfile "1"`. New keywords are additive. A legacy-only Prayfile keeps today’s fan-out (all unbound packages into legacy `output` / `folder` roots). When new-form destinations are present, packages bind only where `pray` appears; unbound legacy `agent` declarations still fan into legacy outputs.

#### render

Declares render policy.

```manifest
render mode: :managed,
  conflict: :fail,
  churn: :minimal
```

Supported fields: `mode`, `conflict`, `churn`, `header`. Parsers MUST reject `section_markers` and `line_endings` (RFC 0034).

---

### 16. Version constraints

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

### 17. Package names

Package names use slash-separated identifiers:

```
namespace/name
```

Examples: `public/base`, `public/webapp`, `sample/security`, `sample/testing`, `sample/ui-kit`

Valid characters: `a-z`, `0-9`, `-`, `_`, `/`, `.`

Package names are case-sensitive. Lowercase is strongly recommended.

---

### 18. Export names

Exports are named package units.

Examples: `webapp-review`, `testing`, `data-layer`, `security-basics`, `project-handoff`

Export names must be unique within a package version.

Good export names: `migration-safety`, `authorization-review`, `system-tests`, `accessibility-basics`

Bad export names: `misc`, `rules`, `all`, `stuff`, `very-important`

---

## Implementation notes

Parsers: `crates/pray-core/src/manifest_parse/`, `package_spec.rs`, `lockfile.rs`. Validation accepts `fail` and rejects other render conflict policies (`manifest_validate.rs`).

Property tests and a local cargo-fuzz harness cover Prayfile, prayspec, and package path validation (changelog 1.6.0).

Ruby and TypeScript CLIs consume `testdata/shared/manifest/` for destination-focused parse slices: `compose-tree-file` and `legacy-target`. Provisioned dest writes follow RFC 0033 (`render_dest` in each CLI).

## Registrar

Prayfile keywords in use: `prayfile`, `source`, `compose`, `tree`, `pray`, `local`, `render`, `group`, plus deprecated `target`, `output`, `agent`, `skills`. Marker comment grammar is RFC 0030. Lockfile keys are RFC 0020.

## Unresolved questions

Whether lockfile TOML is the interchange format, or JSON (the schema) is equally valid on disk (RFC 0020).

Whether `symbols` substitution stays in v1.

## Future possibilities

A `prayfile` editor grammar so extensionless files are not guessed as Ruby or TOML.
