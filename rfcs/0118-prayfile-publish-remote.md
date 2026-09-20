# RFC 0118: Prayfile publish remotes

- Feature Name: prayfile-publish-remote
- Type: Standards Track
- Status: Experimental
- Created: 2026-09-18
- Author: Andrei Makarov
- Relates: RFC 0010, RFC 0040, RFC 0060, RFC 0117
- Requires: RFC 0010, RFC 0040, RFC 0060

## Summary

Prayfile MAY declare named publish remotes. `pray publish` uses those remotes when dest flags are omitted, and publishes only path-owned packages. CLI dest flags MUST match a declared remote when any remote exists.

## Motivation

`pray publish` required `--root` or `--server` on every run. A mixed consumer and publisher Prayfile also packed every resolved dependency into that dest. Operators mistype dest paths.

## Guide-level explanation

```
prayfile "1"
source "amkisko", git: "https://example.com/prayers.git", distribution: "prayers"
source "local", path: "guidance"
publish "prayers", path: "prayers"
publish "public", "https://prayers.example"
compose "AGENTS.md" do
  pray "local/project"
  pray "amkisko/working-rules", "~> 2.4"
end
```

`pray publish` writes `local/project` to `prayers/` and to the server. `pray publish --to prayers` writes only the path remote. `--root` and `--server` remain and MUST equal a declared remote. `pray publish --dry-run` prints names and dests and writes nothing.

An optional block names the package set:

```
publish "prayers", path: "prayers" do
  pray "local/project"
end
```

`yank`, `serve`, and `token` accept `--to NAME` for a path remote. `pray repo init` adds `publish "prayers", path: "prayers"` when a Prayfile exists and has no publish statement.

## Reference-level explanation

Key words follow RFC 2119.

`publish NAME, path: "DIR"` or `publish NAME, "URL"` is a top-level Prayfile statement. NAME MUST be unique among publish remotes. Exactly one dest: `path:` or a positional URL. `path:` MUST be project-relative (RFC 0033). URL MUST be `https://`, `http://`, `pray+ssh://`, or `ssh+pray://`. Implementations MUST reject `source` URLs as implicit push dests. Tokens and signing-key paths MUST NOT parse on this statement.

A `do ... end` block MAY list `pray "package"` names. Those names MUST already be declared and MUST be path-owned. Path-owned means `path:` on the declaration or a path-kind source (RFC 0117). Git, registry, tarball, OCI, and `pray+ssh` packages MUST NOT publish.

An empty block list means the default set: every path-owned declared package. Transitive dependencies MUST NOT publish. `pray package` uses the same default set.

Canonical JSON field `publish_remotes` is an array of `{name, path?, url?, packages}`. Implementations MUST omit an empty array from manifest hash input.

CLI dest selection:

1. No remotes: require at least one `--root` or `--server` (RFC 0040). `--to` is a usage error.
2. Remotes present, no dest flags: use every remote.
3. `--to NAME` (repeatable): use those remotes. Unknown NAME is a usage error.
4. `--root` or `--server`: each value MUST match a remote path or URL. Relative `--root` matches `path:` after `./` strip. Absolute `--root` matches `project_root/path`.
5. `--to` together with `--root` or `--server` is a usage error.

`yank`, `serve`, and `token` with `--to NAME` MUST resolve a path remote. A URL-only remote is a usage error for those commands. When remotes include exactly one path remote and `--root` is omitted, those commands MAY use it.

`pray publish --dry-run` MUST print each selected package name and dest and MUST NOT write the distribution tree, upload, or record a revision.

## Security considerations

A declared remote is a write dest. Matching CLI dests to the allowlist prevents retargeting to an undeclared host. Secrets stay in env and `--signing-key`. Path remotes stay inside the project root.

## Registrar

Prayfile keyword `publish`. Canonical field `publish_remotes`. CLI flags `--to` and `--dry-run` on `publish`; `--to` on `yank`, `serve`, and `token`.

## Drawbacks

A catalog that republished non-path packages with one command must list them with `path:` or stop. Consumer Prayfiles without remotes still require dest flags.

## Rationale and alternatives

`source` stays consume-only. `distribution:` stays git catalog subdir. Dest on prayspec would not fix the project-scoped command. User config is not shared with CI. Defaulting `--root` to `./prayers` without a field would keep silent writes.

## Prior art

Cargo `package.publish` is an allowlist of registry names. npm `publishConfig.registry` is in package.json. Tokens stay out of both manifests.

## Unresolved questions

Whether `sync --to` should share the same remotes. Whether a later prayspec `publish = false` is needed besides the path-owned default.

## Future possibilities

Machine remap of a named remote, the way `[local.source]` remaps install.
