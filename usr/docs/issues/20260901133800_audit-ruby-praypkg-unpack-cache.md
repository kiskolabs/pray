# Audit radius ruby praypkg unpack and cache ready

## Participants

Andrei Makarov

## Decisions

Audit covers the 1.9.2 Ruby unpack and cache-ready changes plus nearby install cache, archive, changelog, and implied-source surfaces. No behavior change from this note alone.

## Effects

Findings ranked below. Highest residual risk from the first pass (path-unsafe tar unpack, in-place cache install, implied source gap) was addressed on patch/fix-ruby-praypkg-unpack-cache.

Unused tempfile require removed from archive.rb. Root CHANGELOG 1.9.2 bullets rewritten away from Encoding.default_internal naming. Safe unpack, staging install, and Ruby implied source shipped with specs.

## Next

Publish 1.9.2 after review. Optional later: share one archive fixture corpus across Rust, Ruby, and TypeScript for escape and size cases.

## Source

patch/fix-ruby-praypkg-unpack-cache
rubygems/pray-cli/lib/pray/archive.rb
rubygems/pray-cli/lib/pray/registry.rb
crates/pray-core/src/package_archive.rs
crates/pray-core/src/registry_cache.rs
npmjs/pray-cli/src/archive/praypkg.ts
usr/docs/changelogs/20260901113000_ruby-praypkg-unpack-and-cache-ready.md
usr/docs/issues/20260901114500_ruby-implied-source-parity.md
CHANGELOG.md
rubygems/pray-cli/CHANGELOG.md

## Findings

Severity high. Confidence high. Location rubygems/pray-cli/lib/pray/archive.rb unpack_praypkg and npmjs/pray-cli/src/archive/praypkg.ts. Kind observed. Why it matters: Rust unpack_praypkg rejects parent escape, symlinks, duplicate paths, and size ceilings; Ruby shells out to tar -xf without those checks, so a hostile .praypkg that passes hash or signature can write outside the cache directory. Smallest fix: list tar entries, validate each path with PathSafety, then extract only safe members, or unpack to a private staging tree and copy allowed files. Deeper fix: share one archive contract and tests across Rust, Ruby, and TypeScript.

Severity medium. Confidence high. Location rubygems/pray-cli/lib/pray/registry.rb resolve_registry_package_root and resolve_local_registry_package_root. Kind observed. Why it matters: Ruby mkdir of the final cache path then unpacks in place; on failure the empty directory remains. cache_ready? now returns false so retry works, but Rust installs via staging then rename and deletes staging on error, so a partial unpack cannot look ready and concurrent readers never see a half tree. Smallest fix: unpack under cache_directory with a .staging suffix, rename on success, rm_rf staging on failure. Deeper fix: match registry_cache.rs install_registry_artifact_to_cache exactly.

Severity medium. Confidence high. Location Ruby format_manifest omit_default_sources versus resolve_package_root_with_metadata. Kind observed. Why it matters: format can drop source: when the namespace matches; Ruby resolve then falls through to a local slug directory. Documented in usr/docs/issues/20260901114500_ruby-implied-source-parity.md. Not fixed in 1.9.2. Smallest fix: port implied_source_name into Ruby resolve. Confirming check: shared fixture that omits source: and installs on all three CLIs.

Severity low. Confidence high. Location rubygems/pray-cli/lib/pray/archive.rb require tempfile. Kind observed. Why it matters: tempfile is required and unused after the encoding fix settled on clearing default_internal. Smallest fix: delete the require.

Severity low. Confidence high. Location CHANGELOG.md 1.9.2 and rubygems/pray-cli/CHANGELOG.md 1.9.2. Kind observed. Why it matters: product changelog names Encoding.default_internal and uses a no-longer truncate clause. House writing guidance keeps user-facing release notes on outcomes. Smallest fix: rewrite to Fix Ruby pray install unpack of .praypkg archives from git and registry sources, and Treat empty registry cache directories as missing so install unpacks again. Keep Encoding detail in usr/docs/changelogs only.

Severity low. Confidence medium. Location rubygems/pray-cli/lib/pray/registry.rb cache_ready?. Kind inference. Why it matters: rescue Error swallows every Pray::Error, including integrity and unsupported, and treats them as not ready. That is usually right for retry, but a poison artifact that always fails validation will refetch every install. Confirming check: inject a signature-mismatch artifact into an empty cache path and count network fetches across two installs. Smallest fix: rescue only resolution errors from find_prayspec_file and parse failures, or delete the cache directory when validate_and_unpack fails.

Severity low. Confidence high. Location rubygems/pray-cli/spec/pray/archive_spec.rb and registry_spec.rb cache_ready?. Kind observed. Why it matters: coverage hits the encoding regression and empty-directory path. Missing: failed unpack then second install succeeds without a no-prayspec error; hostile archive path escape; size ceiling. Not futile. Smallest fix: one integration example that stubs unpack failure leaving an empty cache then asserts resolve retries unpack.

Modes skipped: privacy, no person data in this radius; observability, CLI has no long-lived metrics surface here; learned-systems, none; product-surface accessibility layers, terminal changelog only and covered under prose; dependency-audit, no new packages.

Resource and budget: unpack buffers full artifact and full tar in memory via Open3. Same pattern as before the fix. Ceiling unmeasured this run. Inference until a fixture of a large .praypkg is benched for RSS. Trace and identification: no new identifiers; cache paths stay under .pray/cache/registry; no telemetry added.

Boundary and control: external zstd and tar are commanded processes. Encoding mismatch was a wrong-unit interface fault at stdin. Staging rename reduces reported versus physical cache divergence after a failed unpack.
