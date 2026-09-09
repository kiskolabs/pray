# RFC 0071: Project-local `.pray` directory

- Feature Name: project-local-pray-directory
- Type: Standards Track
- Status: Stable
- Created: 2026-09-07
- Author: Andrei Makarov
- Relates: RFC 0030, RFC 0040, RFC 0070

## Summary

A Prayfile project stores cache, optional vendor copies, local hashes, and write recovery under `.pray/` at the project root. The person ignores that directory by default. Hermetic work commits `.pray/vendor/` and still ignores the other children.

## Motivation

RFC 0070 recommended ignoring `.pray/cache/`. Commands that write destinations now create `.pray/write-state` for crash recovery. That directory can hold original destination bytes. A cache-only ignore leaves those recovery files untracked.

## Guide-level explanation

The person adds this line to `.gitignore`:

```
.pray/
```

`pray install` and other commands that open the project write lock may create `.pray/cache/` and `.pray/write-state/`. A finished command removes the recovery journal; the empty `.pray/write-state/` directory can remain. Deleting `.pray/` after a successful command is safe, and the next install rebuilds cache. Reproducible install needs Prayfile and Prayfile.lock, not local state.

Hermetic or air-gapped work commits `.pray/vendor/` and ignores the local-only children:

```
.pray/cache/
.pray/write-state/
.pray/state.json
```

## Reference-level explanation

Key words follow RFC 2119.

`.pray/` is a directory at the project root beside Prayfile. Implementations MUST create the children in this RFC under that directory, relative to the `--path` root.

### Children

`.pray/cache/` holds project-local package cache. Registry layout is RFC 0070. The cache MUST be deletable. Reproducible install MUST succeed after it is deleted when sources remain reachable or vendor copies exist.

`.pray/vendor/` holds copies written by `pray vendor`. Implementations MUST preserve package tree hashes in that tree. The person MAY commit it for offline or archival work.

`.pray/state.json` MAY hold last render hashes, manual-edit detection data, cache hints, local file hashes, and tool discovery. Implementations MUST NOT require it for reproducible install. Deleting it MUST be safe.

`.pray/write-state/` holds interrupted-write recovery. On Unix, commands that open the project write lock MUST create this directory with owner-only mode `0o700`. Files in it MAY include `journal` and `owner`. The journal MAY store original destination bytes. Implementations MUST recover an interrupted journal on the next command that opens that project's write lock. A successful command MUST remove the journal. The empty directory MAY remain. Deleting write-state after a successful command MUST be safe.

`pray clean` MUST remove `.pray/cache`, `.pray/vendor`, and `.pray/state.json` as RFC 0040 specifies. It MUST leave `.pray/write-state` so an interrupted journal survives hygiene.

### Ignore and commit

The person SHOULD ignore `.pray/` in a default repository:

```
.pray/
```

Hermetic repositories that commit vendor SHOULD ignore every other child named in this RFC. Listing only `.pray/cache/` leaves write-state untracked.

Implementations MUST NOT require `.pray/` contents in version control for a default install. The person MUST NOT commit `.pray/write-state/`.

Rendered destinations stay outside `.pray/`. The person usually commits those files because current inference tools read repository-visible files.

## Implementation notes

Rust `pray-core` transaction module writes `.pray/write-state` for install, apply, update, unlock, add, remove, render, plan, and verify. Ruby and TypeScript CLIs share that layout on Unix.

## Security considerations

A write-state journal MAY contain destination bytes, including secrets that those files already held. The Unix directory MUST be owner-only. Path checks MUST refuse symbolic-link escapes from the project root. The person MUST NOT commit write-state.

## Registrar

Project-root paths: `.pray/cache/`, `.pray/vendor/`, `.pray/state.json`, `.pray/write-state/`, `.pray/write-state/journal`, `.pray/write-state/owner`.

## Drawbacks

An empty `.pray/write-state/` directory remains after success. People who already ignore only cache see an untracked directory until they widen the ignore.

## Rationale and alternatives

Ignore the whole `.pray/` directory because every current child except vendor is local to the machine. Vendor is the hermetic exception, listed by child when the person commits it.

Rejected: keep cache-only ignore and treat write-state as an operator surprise. Rejected: `pray init` rewriting `.gitignore` in this RFC. Rejected: deleting the empty write-state directory on every success, which races with a second command in the same project. Rejected: ignoring only `journal`, which still leaves `owner` and the directory untracked.

## Prior art

Bundler `vendor/bundle` versus machine cache. Cargo `target/`. npm `node_modules/` as the default ignore with an optional committed vendor tree.

## Unresolved questions

Whether `pray init` should add `.pray/` to `.gitignore` when the file exists.

Whether Windows recovery uses the same write-state journal as Unix.

## Future possibilities

`pray init` MAY write or merge a `.pray/` ignore line. A later RFC MAY specify the journal record format if a second consumer needs it.
