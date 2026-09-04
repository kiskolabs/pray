# Registry cache layout parity

## Participants

Andrei Makarov

## Decisions

The claimed two-layout split is real in current source. It is a polyglot cache-parity defect, not a lockfile or single-CLI install correctness bug.

No RFC names an on-disk registry cache schema. RFC 0070 documents `.pray/cache/` and says the Rust CLI is the reference when runtimes disagree. RFC 0020 forbids cache paths in the lockfile. RFC 0040 only says cache path follows OS conventions.

Later pass: do not copy the current Rust hash-first path. Target layout is identity first, source last:

.pray/cache/registry/<namespace>/<name>/<version>/<source hash>

That matches RFC 0070 packages/sample/base/1.4.3/ and the Bundler/Hex name-first habit. The source hash is only the multi-registry leaf so two sources of the same version can sit side by side. Cache remains gitignored, so a one-time miss after alignment is acceptable.

Hash the source identity (URL or source_key), not the artifact. Tree hash and signature stay a reuse check inside the directory. Reject empty, `.`, `..`, and extra separators in namespace and name so Path join cannot escape the cache root.

## Effects

Rust `registry_cache_directory` writes `.pray/cache/registry/<16-hex>/<package_name>/<version>`. The 16-hex is the first 16 characters of sha256 of the first argument. Remote HTTP and SSH pass the source URL. Local registry builds `source_key:name:version:artifact_hash` and hashes that composite. `PathBuf::join` keeps `/` in the package name as extra path segments, so `sample/base` becomes two directories.

Ruby `registry_cache_directory` and TypeScript `registryCacheDirectory` write `.pray/cache/registry/<namespace-name>/<version>/<16-hex>`. They flatten `/` to `-`. The digest is always sha256 of `source_key:package_name:version:artifact_hash` with `no-artifact-hash` when the artifact hash is missing.

Same project, different CLI: cache miss, extra download, `pray install --offline` reports not cached. Each CLI is internally consistent. Vendor path still flattens `/` to `-` in all three languages.

Worked example for source `https://registry.example/index.json`, package `sample/base`, version `1.4.3`, artifact hash 64 a-bytes:

- Rust remote: `.pray/cache/registry/5c6a435144a90f51/sample/base/1.4.3`
- Ruby and TypeScript: `.pray/cache/registry/sample-base/1.4.3/55b13113f083d694`

No GitHub issue and no existing usr/docs issue records this layout split. Nearby notes cover unpack staging and integrity fail-closed, not path order.

Implementation pass: Rust, Ruby, and TypeScript now use the identity-first path and the same source-key hash. `pray clean --unused` validates the complete lockfile before pruning unreferenced registry entries, abandoned staging paths, and legacy layouts without following symbolic links. It preserves Git caches, vendor output, project state, and global caches.

RFC 0070 and RFC 0040 now carry the cache path and cleanup contracts. Shared and runtime-specific regression tests cover parity, unsafe identities, retained and stale paths, malformed or missing lockfiles, and symbolic-link safety. Release validation is recorded in `usr/docs/changelogs/20260904081940_registry-cache-cleanup.md`.

## Next

No open implementation actions remain for registry cache parity or project-local unused cache cleanup. Vendor slash flattening and Git cache pruning remain unchanged by decision.

## Source

crates/pray-core/src/registry_cache.rs
crates/pray-core/src/registry.rs
crates/pray-core/src/registry_ssh.rs
rubygems/pray-cli/lib/pray/registry.rb
npmjs/pray-cli/src/registry/cache.ts
rfcs/0070-reference-implementation.md
rfcs/0020-resolve-and-lock.md
rfcs/0040-cli-surface.md
usr/docs/issues/20260901133800_audit-ruby-praypkg-unpack-cache.md
usr/docs/issues/20260901135528_engineering-audit-auth-integrity-parity.md
usr/docs/changelogs/20260904081940_registry-cache-cleanup.md
