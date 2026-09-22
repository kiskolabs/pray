# Bundler, Cargo, and Mix findings for pray dependency workflows

## Participants

Engineering review of pray reference CLI against Bundler, Cargo, and Mix git and lockfile patterns.

## Decisions

Implement findings in priority order. Phase 1 complete: lock git revision per source in Prayfile.lock and checkout pinned commits during resolve (including file:// git repos via cache).

| Priority | Finding | Status |
|----------|---------|--------|
| 1 | Lock git revision on [[source]] | done |
| 2 | Resolver prefers locked package versions | done |
| 3 | Vendor fallback in resolve; fix --offline | done |
| 4 | pray unlock package | done |
| 5 | Configurable subdir on git sources | done |
| 6 | Local override config (config.toml) | done |
| 7 | Bare shared cache + materialize | done |
| 8 | Sparse fetch | done |
| 9 | gix migration for fetch | deferred |

## Effects

Phase 1 adds optional `revision` on `[[source]]` in Prayfile.lock for git sources. Resolve checks out the locked commit instead of always tracking remote HEAD. Fresh installs record `git rev-parse HEAD` after fetch.

Phase 2 adds lockfile-aware package version selection, vendor fallback for offline resolve, `pray unlock <package>`, `subdir:` on git sources, `~/.config/pray/config.toml` local overrides, shared bare git cache under `~/.cache/pray/git`, and sparse checkout when subdir is set.

## Next

- gix migration for fetch (deferred until cache design stabilizes)
- SPEC updates for revision, subdir, unlock, and config.toml when cutting a release

## Source

Prior conversation comparing pray git integration to Bundler (subprocess git, bare cache), Cargo (SHA-pinned lock, gix fetch, vendor), and Mix (sparse/subdir, deps.unlock).
Upstream: crates/pray-core/src/resolve.rs, crates/pray-core/src/lockfile.rs, crates/pray-core/src/config.rs, crates/pray-core/src/resolve_context.rs, crates/pray-core/src/registry.rs, SPEC.md sections 31–32 and 35.
