# RFC 0033: Provisioned destination write safety

- Feature Name: provisioned-destination-safety
- Type: Standards Track
- Status: Experimental
- Created: 2026-09-02
- Author: Andrei Makarov
- Relates: RFC 0010, RFC 0020, RFC 0030, RFC 0031, RFC 0040
- Requires: RFC 0010, RFC 0020, RFC 0031

## Summary

Exclusive `file:` and `tree:` leaves refuse to clobber unmanaged bytes, refuse symlink destinations, record each leaf in `Prayfile.lock`, and prune a dropped leaf only when on-disk bytes still match the locked hash. `pray plan` lists every such path. Marker dialects are out of scope.

## Motivation

Home-as-root (`pray --path $HOME` or `PRAY_PATH`) already selects a project tree. Exclusive `file: ".zshrc"` already writes unmarked bytes. Today `fs::write` overwrites a regular file and follows a symlink. `Prayfile.lock` has no provisioned-path array, so remove cannot implement RFC 0031 step 4 without tagging dest files. Plan groups sibling leaves and compares dest to raw source bytes.

## Guide-level explanation

```manifest
prayfile "1"
pray "owner/shell-rc", "~> 1.0", file: ".zshrc"
```

`pray --path "$HOME" plan` prints `.zshrc would be written` (or unchanged / would be updated). It does not print `~/.zshrc`. A leading `~` in a destination string is a manifest error. Source and local package paths may begin with a literal `~` directory name.

First apply writes when the dest is missing or already equal to reconstructed expected bytes (UTF-8 after `((pray:…))`, else raw). If `.zshrc` exists with other bytes, apply fails. If `.zshrc` is a symlink, apply fails.

A later apply that still owns that path in the previous lock may replace dest when on-disk bytes still match the locked content hash. If the operator edited the dest, apply fails until they restore or remove the file.

Plan applies the same ownership check. It fails with the same refusal as apply when a provisioned path is unmanaged, edited, a symlink, or not a regular file.

`[[provisioned]]` records path, content hash, package, and export. Remove deletes a dest only when that hash still matches. User-edited dests stay. Undeclared siblings under a `tree:` root stay.

Compose still uses HTML comment markers. Compose of `.zshrc` or `.json` is a render error that names `file:` as the unmarked path (RFC 0108). Native include of an unmarked `file:` dest is the home splice. `$HOME` or `/` as `--path` is a normal project root, not a tilde product. `/` as root is allowed and widens blast radius.

## Reference-level explanation

Key words follow RFC 2119.

Project-relative destinations MUST reject empty, absolute, parent, and leading-`~` strings. `~` is a name, not home expansion. The leading-`~` rule applies to destination fields, including `file:`, compose outputs, tree folders, and resolved provisioned leaves. It MUST NOT reject a source or local package path solely because its first component begins with `~`.

Before reading, writing, or pruning a destination, implementations MUST inspect every existing path component below the selected project root without following it. A symlink or reparse-point ancestor MUST fail with a render error. Implementations MUST repeat this check after creating parent directories and immediately before creating a destination leaf.

Implementations MUST inspect the final path component without following it. A symlink dest MUST fail with a render error. A non-file dest MUST fail. On systems with a no-follow open flag, compare, hash, and update MUST use the same no-follow file descriptor and verify that descriptor is a regular file. Creation MUST be exclusive. Implementations without an equivalent primitive MUST re-check the final component immediately before mutation and remain Experimental. Package-archive symlink reject is unchanged (RFC 0050). Compose dests MUST use the same gate and preserve unmarked text around recognized managed spans.

Reconstructed expected bytes are `expected_provisioned_bytes`: UTF-8 with symbols substituted, otherwise raw source bytes.

Refuse-clobber for each planned exclusive `file:` or `tree:` leaf:

1. Missing dest: write.
2. Regular dest whose bytes equal expected: skip write (adopt).
3. Regular dest listed in the previous lock `[[provisioned]]` whose on-disk hash equals that record's `content_hash`: write (managed update).
4. Otherwise: render error. Do not write `dest.bak`.

`Prayfile.lock` MAY omit `provisioned` when empty. Each leaf MUST record:

```toml
[[provisioned]]
path = ".zshrc"
content_hash = "sha256:..."
package = "owner/shell-rc"
export = "zshrc"
```

`content_hash` is `sha256:` of expected bytes. Verify MUST report a missing planned dest by path. Verify MUST fail a provisioned dest that is a symlink or whose bytes differ from expected.

On package remove or export drop, validate each previous `[[provisioned]].path` as a destination before joining it to the project root. Delete a previous-lock dest not in the new plan only if it is a regular file and on-disk hash equals the locked `content_hash`. Skip user-edited dests. Do not follow a symlink to unlink a target. Do not tag dest files with origin comments.

`pray plan` MUST print every provisioned path that would change. It MUST NOT group siblings. Change detection MUST compare dest to reconstructed expected bytes and apply the same ownership refusal as materialization.

Materialization MUST finish all destination writes and hash-gated pruning before replacing `Prayfile.lock`. A destination failure MUST leave the previous lock bytes intact so a retry retains the prior ownership ledger.

Provisioned scripts MUST keep the mode of a regular file write in this RFC (no added execute bit). Prefer `pray trust set-require-signed-packages` in operator notes; that flag already exists.

## Implementation notes

Rust: `render_dest.rs` is the write gate for provisioned leaves. `write_rendered_targets_with_previous_lockfile` passes the previous lock into materialize. `apply_report.rs` lists each path. Schema: `schema/lockfile.schema.json` optional `provisioned` array. Ruby: `render_dest.rb`. TypeScript: `render/dest.ts`.

## Security considerations

Home as project root is the selected tree, not an extra-repo allow. A symlink selected as the project root is allowed; the ancestor rule begins below that selected root. Inert packages stay. Symbols stay explicit maps. Plan MUST list every provisioned path that would be written.

## Registrar

Lockfile array `[[provisioned]]` with keys `path`, `content_hash`, `package`, `export`.

## Drawbacks

First apply onto a populated home file fails until the operator removes it or the bytes already match. Plan fails on the same path instead of presenting an update. Old locks without `provisioned` cannot prune or do managed update until the next successful apply records the array.

## Rationale and alternatives

APM hash-gated cleanup and adopt-if-identical, without `--force` or `dest.bak`. RFC 0031 origin tags on dest files are replaced by the lock ledger. Chezmoi `modify_`, Copier merge, AgentSync symlink farms, and APM named start/end markers stay out.

Rejected: user format plugins; tilde expansion; Bundler walk-up into HOME; compose dialects in this RFC; `ids/0032`.

## Prior art

microsoft/apm `is_content_identical_to_source` and hash-gated `cleanup.py`. dallay/agentsync `symlink_metadata` plus revalidate before mutate. RFC 0031 remove step 4.

## Unresolved questions

Which Windows handle flags and reparse-point checks are required before Stable. Whether an empty parent directory after prune should be removed.

## Future possibilities

RFC 0108 file-as-fragment. Per-destination compose header. Fail-closed compose of JSON. Marker dialect RFC 0032 if scheduled. Named slots. Sidecar spans.
