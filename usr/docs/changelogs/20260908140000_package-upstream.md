# Package upstream

## Participants

Andrei Makarov

## Decisions

A path package may declare one upstream in its prayspec. The catalog lock stores the resolved version and hashes. pray update refreshes a clean replica by replacing content files and keeping fork identity. Upstream is not a compose dependency.

## Effects

RFC 0114 is Experimental. All three runtimes reject duplicate upstream declarations, resolve the locked identity and source on install, and verify the locked tree and artifact hashes. Rust update replaces or three-way merges path trees inside the project transaction. It validates content paths and archive-sized file, tree, and entry limits before mutation. The rewritten fork prayspec preserves every supported field and local-only content entry. Ruby and TypeScript refuse a path-fork update before writing because they do not implement path-tree refresh.

## Next

Adding upstream fields to public Rust structs is source-incompatible for callers that construct those structs. Release the Rust API in the next major version or redesign those additions before publish.

Later pass 20260908174500: this work ships as 1.13.0. See usr/docs/issues/20260908174500_prepare-1-13-0-release.md.

Later pass 20260915102100: published registry metadata must not echo spec.upstream. Ruby and TypeScript now refresh path forks. See usr/docs/changelogs/20260915102100_registry-metadata-and-ruby-latest.md.

## Source

RFC 0114
CHANGELOG.md 1.13.0
usr/docs/issues/20260908174500_prepare-1-13-0-release.md
usr/docs/changelogs/20260908174400_tag-1-13-0.md
