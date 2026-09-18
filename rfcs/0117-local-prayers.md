# RFC 0117: Local prayers

- Feature Name: local-prayers
- Type: Standards Track
- Status: Experimental
- Created: 2026-09-16
- Author: Andrei Makarov
- Relates: RFC 0002, RFC 0010, RFC 0011, RFC 0020, RFC 0030, RFC 0040
- Requires: RFC 0011, RFC 0040

## Summary

A consumer project keeps local prayers as packages under a path source directory of the author's choosing. Version and a `.praypkg` are not required until the prayer is packaged or published. `.agents/project.md` stays a compose shortcut for one human-owned file.

## Motivation

Compose of `.agents/project.md` is a short local embed. It does not look like a package, so people do not polish that text into something they can share. Path packages already exist, but `pray prayer init` scaffolds a versioned package in the current directory, and examples use `packages/`. That is the published-gem shape. A project-local gem does not need `gem build`.

A per-package `path:` next to compose `pray "project"` names the same prayer twice. A path source names the folder once.

## Guide-level explanation

```text
guidance/project/
  project.prayspec
  README.md
  exports/project.md
```

```manifest
prayfile "1"
source "amkisko", git: "https://example.com/prayers.git"
source "local", path: "guidance"
compose "AGENTS.md" do
  pray "local/project"
  pray "amkisko/working-rules", "~> 2.0"
end
```

The directory name is the author's. `prayers/` is the default when `pray prayer init` adds a path source. `pray prayer init notes --path guidance` writes `guidance/notes/`.

`pray prayer init` in a directory with a Prayfile writes `<dir>/<name>/` with no `spec.version`. It adds `source "local", path: "<dir>"` when the project has no path source. It declares `pray "<source>/<name>"` once, such as `pray "local/project"`: inside the first `compose` block when one exists, otherwise at the top level, never with `path:`. `spec.name` is that same token. A starter fragment is included.

Without a Prayfile, `pray prayer init` still scaffolds a versioned package in the current directory for a standalone package repository.

`.agents/project.md` inside `compose` remains valid. Prefer a path-source package when the text is meant to grow into a shareable prayer.

`pray package` and `pray publish` fail until `spec.version` is set.

## Reference-level explanation

Key words follow RFC 2119.

A local prayer MUST be a path-source package: a directory under the path source URL with one `*.prayspec` and the files listed in `spec.files`. Implementations MUST NOT write a `.praypkg` for that tree during `pray install`.

`spec.version=` MAY be omitted. Canonical JSON MUST omit an empty version. A missing version and the reserved string `local` MUST satisfy only the default constraint `*`. Any other constraint MUST fail with a message that the package needs `spec.version` or that the constraint must be omitted. `Prayfile.lock` MUST record version `local` for that package. Identity remains `tree_hash`.

`pray package` and `pray publish` MUST fail when `spec.version` is missing or `local`.

When a package name contains no `/`, and exactly one declared source has kind `path`, implementations MUST use that path source even when other sources exist. An explicit `source:` still wins. A namespace that matches a source handle still wins. Several path sources still require a namespace or `source:` on an unqualified name.

The directory under a path source is the last segment after a matching source handle (`pray "local/project"` lives at `<path>/project`). An unqualified name still uses the hyphenated slug of the whole name.

`spec.name` MUST equal the Prayfile package token. Init-created local prayers therefore use `spec.name = "local/project"` when the source handle is `local` and the folder is `project`.

A `pray` token is a local path when it starts with `.` or `/`, or ends with `.md`, `.txt`, or `.markdown`. Any other unqualified name is a package. `pray "project"` inside `compose` is therefore a package named `project`, not a local file.

`pray prayer init [name] [--path DIR]` with a Prayfile MUST write `<dir>/<name>/` relative to the project root. The default name MUST be `project`. The default `DIR` MUST be `prayers` when the project has no path source. When exactly one path source exists and `--path` is omitted, implementations MUST use that source's directory. When `--path` is set, it MUST match an existing path source or become the directory of a newly added `source "local", path: "<dir>"`. Several path sources and no `--path` MUST fail. A `--path` that disagrees with the existing unique path source MUST fail. The name MUST be one path segment. Names matching `v` followed by one or more digits MUST fail; those folders are the distribution layout (RFC 0002). If that directory or its `*.prayspec` already exists, the command MUST fail.

If the Prayfile does not already declare the package, implementations MUST append `pray "<source>/<name>"` once: as a line inside the first `compose` block when one exists, otherwise as a top-level declaration. That line MUST NOT include `path:`.

`pray prayer init` without a Prayfile MUST keep the current-directory package scaffold and MUST write `spec.version`.

The folder `prayers/v1/` remains the distribution checkout from `pray repo init`. When the path source directory is `prayers/`, named prayer directories MAY sit beside `v1/`.

## Implementation notes

Reference CLIs: `prayer init` local mode, omitted `spec.version` parse and render, lock `local`, package and publish refusal, unique path-source inference for unqualified names. Schema: `package.schema.json` does not require `version`.

## Registrar

CLI: `pray prayer init [name] [--path DIR]`. Default path source name `local`. Default path source directory `prayers`. Lockfile package `version` value `local`. Reserved names `v` plus digits.

## Drawbacks

When the path source directory is `prayers/`, that folder also holds `v1/` after `pray repo init`. Authors must not name a prayer `v1`.

## Rationale and alternatives

Rejected: requiring `spec.version = "0.1.0"` for local trees; putting local prayers under `packages/`; replacing `.agents/project.md`; treating git clone of a distribution as the writable overlay (RFC 0114, RFC 0116); a hardcoded `prayers/<name>/`; `pray "<name>", path: "<dir>/<name>"` plus a second `pray "<name>"` in compose.

A lock value of `0` would look like a release. `local` is the recorded stand-in until a version exists.

Unqualified names bind to the unique path source so a git catalog and a local folder can coexist without `source:` on a single-segment name. `pray "local/project"` is the form that names the source when more than one source exists.

## Prior art

Bundler path gems without `gem build`. Rubygems still require a version in the gemspec; this RFC omits that until pack or publish.

## Unresolved questions

Whether a later rename of `spec.name` to a registry namespace should rewrite the Prayfile declaration.

## Future possibilities

A publish flow that copies a local prayer from the path source tree into a distribution `v1/packages` tree after a version is added (RFC 0118).
