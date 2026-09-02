# RFC 0011: Prayspec and package archive

- Feature Name: prayspec-and-package
- Type: Standards Track
- Status: Stable
- Created: 2026-08-18
- Author: Andrei Makarov
- Relates: RFC 0010, RFC 0020, RFC 0050, RFC 0060, RFC 0034
- Requires: RFC 0010

## Summary

A package is a static text tree with a `*.prayspec`, named exports, optional provisioned folders, a `*.praypkg` archive, and a normalized tree hash. Parse is declaration-only.

## Motivation

Independent implementations need one meaning for package bytes, export names, and archive members. Path escape and host-language execution are out of scope.

## Guide-level explanation

Authors write a `*.prayspec` beside `exports/`, optional `folders/`, and adapters. `pray package` builds a `*.praypkg`. Install verifies `artifact_hash` and `tree_hash` (RFC 0050) before unpack.

## Reference-level explanation

This section is the product contract for this concern. Where it disagrees with Implementation notes, Implementation notes record what the reference CLI does today. A follow-on RFC records the gap.

### 19. Package layout

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
  folders/
    code-review/
      README.md
      assets/
        checklist.md
  templates/
    pr-review.md
    incident-report.md
```

Required: `*.prayspec`

Optional: `README.md`, `LICENSE`, `CHANGELOG.md`, `exports/`, `folders/`, `templates/`, `assets/`

`spec.adapters` MAY still parse as a string map. It is unused (RFC 0034). Destination DSL names dest paths. Do not ship adapter TOML to spell markers.

---

### 20. prayspec design

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
    "folders/code-review/README.md",
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
    },
    "code-review" => {
      type: "folder",
      path: "folders/code-review",
      summary: "Application code review checklist"
    }
  }
  spec.templates = {
    "pr-review" => {
      path: "templates/pr-review.md",
      summary: "Pull request review template"
    }
  }
  spec.targets = ["tool_a", "tool_b", "generic"]
  spec.add_dependency "sample/base", "~> 1.4"
  spec.metadata = {
    "prayfile.target.tool_a" => "true",
    "prayfile.target.tool_b" => "true"
  }
end
```

---

### 21. prayspec allowed grammar

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

`skills=` is deprecated and will be removed in version 2. Prefer a `folder` export.

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

### 22. prayspec canonical model

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
    },
    "code-review": {
      "type": "folder",
      "path": "folders/code-review",
      "summary": "Application code review checklist"
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

### 23. Export types

Supported export types:

- fragment: Text fragment rendered into a `compose` / legacy output
- file: Exact file bytes via `pray …, file: "path"` (preferred) or nested under a legacy folder root
- folder: Directory tree provisioned into a `tree` / legacy folder root

`skill` is a deprecated alias for `folder` and will be removed in version 2. Types `template`, `command`, `rule`, `asset`, and `bundle` match no destination role (RFC 0034). Do not invent dest types for them.

Folder exports may declare `only: [...]` or `except: [...]` relative paths to provision a subset of the tree. `default_path` on a `file` export is a publisher hint only; the consumer `file:` path wins.

---

### 24. Provisioned folders

A `folder` export is a directory tree copied deterministically into a `tree` destination (or a legacy target `folder` root).

A `file` export with `pray …, file: "SECURITY.md"` writes the export to that path at the project root (or relative path given), after `((pray:…))` substitution for UTF-8 text. Legacy fan-out without `file:` still copies under `<folder-root>/<export-name>/`.

Example:

```ruby
tree ".agents/skills" do
  pray "amkisko/engineering-audit", "~> 2.0"
end

pray "amkisko/community-security", "~> 1.0", file: "SECURITY.md"
```

Legacy equivalent:

```ruby
target :agents do
  output "AGENTS.md"
  folder ".agents/skills"
end
```

`skills` in a target block is a deprecated alias for `folder` and will be removed in version 2.

Optional support files may live under `assets/`, `templates/`, or `examples/` inside the folder export.

Two packages must not install the same folder path.

---

### 25. Package payload rules

V1 packages are data packages.

Allowed package contents: Markdown, TOML, JSON, YAML, plain text, templates, declared assets, images/diagrams if useful for review checklists, scripts as inert assets only

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

### 26. Package archive

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

### 27. Normalized package tree hash

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

## Implementation notes

Parsers and archive code live in `package_spec.rs`, `package_archive.rs`, and unpack tests in changelog 1.6.0.

## Security considerations

Archive members MUST NOT escape the extract root. Implementations MUST NOT execute package code.

## Registrar

Prayspec fields: name, version, summary, description, authors, exports, adapters, and related declaration keys in the grammar above. `spec.skills` and export `type: "skill"` still parse and warn; they are removed in version 2.

## Unresolved questions

Whether JSON Schema or this RFC wins when they disagree on field presence. Recommendation: schema plus fixtures win for field presence; this RFC wins for algorithms until RFC 0100 is Stable.
