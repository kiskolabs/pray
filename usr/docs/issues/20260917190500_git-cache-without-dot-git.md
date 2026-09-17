# Git catalog cache does not need a working-tree .git for prayer files

## Participants

Andrei Makarov

## Decisions

No product change in this pass. Research only.

Prayer files used for resolve and render do not need a .git directory. After a git source is cloned, install reads v1/packages metadata and a .praypkg, unpacks that archive into .pray/cache/registry, and renders from the unpacked tree. That registry cache is already git-free.

The project git catalog cache still keeps a git object store because the current design uses git after clone: refresh fetch, pinned checkout, blobless materialize, worktree share, global bare seed, rev-parse HEAD for the lock pin, and git verify-commit plus git log %GK/%GF when trust policy requires a signed commit.

Deleting .git from a project catalog cache today is not a no-op. ensure_git_repository treats a path without a .git directory as stale, removes it, and clones again.

## Effects

Claim under review: cloned or cached prayers do not need .git.

Split:

1. Unpacked prayer packages and rendered destinations do not contain .git. Outcome: supported. resolve_local_registry_package_root copies artifact bytes from the distribution root and unpacks into .pray/cache/registry. Render reads that tree.

2. The git catalog cache can drop .git and still install a locked revision from files already on disk. Outcome: partially supported. The files needed for one install are the distribution tree, not git metadata. The running code will not reuse that tree without .git. Blobless catalogs also keep unused artifact blobs out of the worktree and fetch them later with git checkout HEAD -- path, which needs the object store.

3. RFC 0060 makes git optional for distribution. Outcome: supported. A distribution point is a static file tree. HTTP, path, and tarball sources never clone. Git is one transport to obtain that tree at a revision.

4. Trust policy can enforce signed commits without keeping .git after the check. Outcome: partially supported. git verify-commit reads commit objects from the repository, not the worktree. The check can run at fetch time and then the object store can be dropped if the pin and hashes are recorded. The current gate runs on every ensure and skips when .git is not a directory.

Measured 2026-09-17 on this project's cache of github.com/amkisko/prayers.git at lock revision 42182fbd641e99e9095c4cca83a6d3c7cd43e8fe:

- URL-only project cache 4.2 MiB total
- .git 2.8 MiB
- prayers/ 1.2 MiB, of which artifacts 1.0 MiB and packages 236 KiB
- leftover root files from cone-mode sparse checkout: README, AGENTS.md, Makefile, Prayfile, .gitignore, and similar. Git cone mode always includes tracked files in the repository root. Official git-sparse-checkout docs, fetched 2026-09-17: for a cone directory, paths immediately under leading directories including the toplevel directory are also included. Outcome: supported.

That URL-only clone predates blob:none. Its sparse-checkout list is still prayers, so artifacts sit in the worktree. A same-day subdir worktree has the later cone prayers/v1/packages. That worktree .git is a gitdir file pointing at the URL-only object store.

Global cache under the default Unix path is a bare git db per clone URL. Sample entries are HEAD plus objects, no worktree. About 885 directories, about 108 MiB together. Average size matches small shallow bare clones, including file:// test catalogs if PRAY_CACHE was unset.

What current code uses .git for, with source:

- clone --depth 1 --filter=blob:none --sparse, then sparse-checkout cone. crates/pray-core/src/resolve_git_clone.rs
- fetch --depth 1 --filter=blob:none origin and reset --hard on update. resolve_git.rs refresh_git_worktree
- checkout of the lock revision. resolve_git.rs checkout_git_revision
- git rev-parse HEAD written to Prayfile.lock [[source]].revision. RFC 0020 unresolved question names that pin. This project's lock has it.
- materialize of a missing artifact blob. resolve_git_materialize.rs
- worktree add for a subdir source sharing one object store. resolve_git_ensure.rs
- clone --bare of the project cache into the global cache. resolve_git.rs mirror_git_cache_to_global
- git verify-commit HEAD and git log -1 --format=%GK and %GF. RFC 0060 section 29.6. client_trust/enforce.rs and client_trust/git.rs

gate_git_source returns success when .git is not a directory. Linked worktrees store .git as a file, so a subdir checkout skips signed-commit enforcement. TypeScript gitDirectory has the same isDirectory check. is_git_checkout uses exists, which is the correct worktree test.

Prior art, fetched 2026-09-17:

- Cargo keeps a bare db under git/db and checkouts under git/checkouts. The db is the git object store. Checkouts are the crate source. Cargo Home docs. Outcome: supported for the split. Cargo's checkout is the package. Pray's git checkout is a catalog; the package is the unpacked .praypkg.
- Nix fetchgit defaults leaveDotGit to false and hashes the tree. Outcome: supported as a model that strips .git after clone.
- Go module cache stores a zip of the tree, not a git repository. Outcome: supported as another git-free install cache.
- GitHub Blog on partial clone, 2020-12-21: blob:none still downloads commits and trees into .git and fetches blobs on demand. Shallow clone is for clone-and-delete, not for a long-lived fetch cache. Outcome: supported.

usr/docs/issues/20260916225000_git-free-distribution.md already decided git is optional for publish and install when the source is HTTP. That does not remove git as a source kind.

usr/docs/issues/20260705204000_bundler_cargo_mix_findings.md already chose a shared bare git cache plus materialize. Global cache matches that. Project cache still materializes a git worktree instead of a git-free tree.

If .git is removed from a project catalog without a design change:

- next ensure deletes the tree and clones again
- blobless materialize cannot fetch a newly selected artifact
- update cannot fetch; it reclones
- verify-commit cannot run on that path
- global seed cannot git clone --bare from it

A design that matches the claim without losing those jobs:

- keep git objects only in the global bare db
- materialize the distribution tree (v1/packages and used artifacts, or git archive of that prefix) into the project cache without .git
- record the revision in the lock, which already happens
- run verify-commit against the bare db at fetch time
- on update, fetch in the bare db, then replace the git-free tree

That is a behavior change and needs tests first. Cone-mode root files are a smaller separate cleanup: non-cone patterns can hide README and .gitignore without dropping the object store.

## Next

Decide whether project git cache should become a git-free distribution tree while the global cache stays the bare object store.

If yes: tests that a locked install reuses a catalog directory with no .git, that update fetches in the bare db, that blobless artifact materialize still works, and that require_signed_commit runs on the bare db including subdir sources.

Fix gate_git_source to treat a .git file as a checkout so worktrees do not skip trust. Independent of stripping .git.

Optional: non-cone sparse patterns so root files are not checked out. Independent of stripping .git.

Do not delete .git from existing caches until the reuse path exists.

## Source

crates/pray-core/src/resolve_git.rs, resolve_git_ensure.rs, resolve_git_clone.rs, resolve_git_materialize.rs, resolve_git_lookup.rs, registry.rs, registry_cache.rs, client_trust/enforce.rs, client_trust/git.rs

npmjs/pray-cli/src/git/cache.ts

rubygems/pray-cli/lib/pray/git_cache.rb

rfcs/0060-distribution.md section 29.6

rfcs/0020-resolve-and-lock.md

rfcs/0070-reference-implementation.md cache layout names registry packages, not git checkouts

usr/docs/issues/20260916225000_git-free-distribution.md

usr/docs/issues/20260917154800_multi-source-prayfile-efficiency.md

usr/docs/changelogs/20260917174500_git-blobless-clone.md

usr/docs/changelogs/20260917171000_git-worktree-share-and-import-repo.md

https://git-scm.com/docs/git-sparse-checkout cone mode toplevel files, fetched 2026-09-17. Outcome: supported.

https://git-scm.com/docs/git-verify-commit, fetched 2026-09-17. Outcome: supported.

https://github.blog/open-source/git/get-up-to-speed-with-partial-clone-and-shallow-clone/, fetched 2026-09-17. Outcome: supported.

https://doc.rust-lang.org/cargo/guide/cargo-home.html git/db versus git/checkouts, fetched 2026-09-17. Outcome: supported.
