# How Bundler and Cargo handle git sources

## Participants

Andrei Makarov

## Decisions

Research only. No product change in this pass. Compare Bundler and Cargo git cache, lock pins, missing-object fetch, shallow clones, and offline against issue 31 and pray's current tree.

Shared pattern in both tools:

1. Lock a commit SHA. Install honours that SHA. Update is the command that re-resolves a branch or tag.
2. Keep a shared object store separate from a revision checkout.
3. If the lock SHA is already in the object store, do not fetch.
4. If the lock SHA is missing from an existing store, fetch it. That is still install, not update.
5. Offline refuses a fetch. Offline succeeds when the object is already local.

Pray's issue 31 failure was step 4 gated on refresh instead of on object presence. The 20260918 fix matches this pattern for pinned checkout. Residual gaps below.

## Effects

Cargo, sources fetched 2026-09-18.

Layout. The Cargo Book Cargo Home page: git/db is a bare clone per remote; git/checkouts holds a working tree per commit, named by a short object id. Multiple checkouts share one db. Official docs: https://doc.rust-lang.org/cargo/guide/cargo-home.html

Lock. Specifying Dependencies: Cargo writes the git commit into Cargo.lock at first resolution and checks for updates only on cargo update. https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html

Skip fetch when present. GitSource::fetch_db: if Revision::Locked(oid) and GitDatabase::contains(oid), return that db with no network. contains is revparse_single on the oid. Comment on the function: This won't fetch anything if the required revision is already available locally. Source: nightly rustc rendering of src/cargo/sources/git/source.rs, fetched 2026-09-18.

Fetch when missing. The same match's last arm: locked revision with a db that does not contain it still fetches into that db. Offline in that arm bails: can't checkout from url: you are in the offline mode. Outcome: supported. This is the issue 31 case.

How it fetches a SHA. sources/git/utils.rs fetch: GitHub fast path may yield NeedsFetch(rev), then refspec +oid:refs/commit/oid. If indeterminate, fetch heads and tags and hope the commit is reachable. Shallow gitoxide path can fetch +rev:refs/remotes/origin/HEAD. Offline at the start of fetch: attempting to update a git repository, but offline was specified. Source: cargo utils.rs on master via the 2026-09-18 snapshot used in this run.

Default clone depth. Cargo Home describes a bare clone with no depth-1 default. Shallow git deps live under -shallow suffixes and need -Zgitoxide. rust-lang/cargo PR 11840. Outcome: supported that default git/db is not a long-lived depth-1 cache. Pray's project cache is shallow by design, so missing ancestors are more common.

Offline and vendor. cargo fetch downloads lock deps so later cargo build --offline can run. cargo vendor copies git and registry sources into a directory. --frozen is --locked plus --offline. --locked is lockfile change, not git fetch. https://doc.rust-lang.org/cargo/commands/cargo-fetch.html and cargo-vendor.html

Package vs catalog. Cargo's checkout is the crate source the compiler reads. Pray's git checkout is a catalog; packages unpack from .praypkg. Same split already recorded in usr/docs/issues/20260917190500_git-cache-without-dot-git.md. Outcome: supported.

Bundler, sources fetched 2026-09-18.

Layout. Source::Git cache_path is a shared git cache under Bundler.user_cache/git or bundle_path/cache/bundler/git. install_path is a checkout of one revision. GitProxy.copy_to clones --no-checkout from that cache into install_path, then reset --hard to the revision. ruby-doc Bundler 3.4 Git and GitProxy. Outcome: supported.

Lock. Gemfile.lock GIT block has revision:. Later installs fetch exactly that commit, even if the branch has moved on. https://guides.rubygems.org/gemfile-lock/ fetched 2026-09-18. Outcome: supported.

Skip fetch when present. GitProxy.checkout returns immediately if has_revision_cached?. That is git cat-file -e on the locked revision in the cache path. Outcome: supported.

Fetch when missing. If not cached, clone --bare --no-hardlinks, optionally --depth 1 --single-branch. extra_clone_args skips --branch when a lock revision is already known. Then git_remote_fetch. clone_needs_unshallow? is true when path/shallow exists and either a full clone is required or @revision is set and differs from HEAD. That unshallow is Bundler's answer to a depth-1 cache that does not contain the lock SHA. Outcome: supported. ruby-doc 3.4 GitProxy clone_needs_unshallow? and checkout.

Fetch of a SHA. refspec for a commit is commit:refs/commit-sha. copy_to may git fetch that ref from the cache into the checkout, and for git <= 2.13.7 sets uploadpack.allowAnySHA1InWant on the cache. extra_fetch_args passes --depth when shallow. Outcome: supported.

Offline. Git#fetch rescues GitError and, if allow_offline_install, warns Using cached git data because of network errors. bundle install --local uses gems already on disk. Not the same flag as cargo --offline; same idea: no network, fail or use cache. Outcome: partially supported as a family, not as identical flags.

Local override. bundle config set local.GEM_NAME path. Bundler requires a Gemfile branch, checks the local branch matches, and checks the lock revision exists in the local repo so you fetch remotes before locking a commit that only exists on your machine. https://guides.rubygems.org/git/ fetched 2026-09-18. Outcome: supported. Pray has no equivalent local git override.

Git gems have no CHECKSUMS digest in Gemfile.lock because there is no .gem blob. Outcome: supported from the same lockfile guide.

What this means for pray.

Issue 31: Bundler and Cargo both fetch a missing lock SHA into an existing cache. They do not tell the operator to drop --locked. Cargo names offline when the object is absent. Bundler unshallows a shallow cache when the lock SHA is not HEAD. Pray's 20260918 change fetches origin revision with depth 1 unless offline. That is closer to Cargo's SHA refspec and Bundler's extra fetch than to unshallow.

Cache shape: both keep a shared object db and a per-revision worktree. Pray keeps a per-project worktree that is also the object store, plus a global bare seed. Cargo's checkout is disposable (CI caches git/db, not checkouts). Pray's catalog worktree is the install input until packages unpack.

Shallow: Bundler uses depth 1 when git can fetch unreachable SHAs, then unshallows or extra-fetches the pin. Cargo's default db is not shallow. Pray clones --depth 1 --filter=blob:none for every catalog, so ancestor misses are the common path.

Offline: Cargo fails closed if the oid is missing. Pray now matches that for pinned checkout. A missing pray cache still clones during --offline. Cargo would also need the db present; cargo fetch is the prepare step. bundle cache / vendor/cache is Bundler's prepare step.

file:// fallback in pray (resolve_git_package_root reads the origin worktree when cache checkout fails) has no Cargo equivalent. Bundler local override is explicit config, not a silent fallback.

## Next

Git-free project catalog and unshallow pin fetch shipped in usr/docs/changelogs/20260918213000_git-free-catalog-and-unshallow.md.

Clone of a missing cache is gated on --offline. file:// origin worktree is no longer read after a pin checkout fails. See usr/docs/changelogs/20260918210000_offline-git-clone-and-file-origin.md.

## Source

https://doc.rust-lang.org/cargo/guide/cargo-home.html fetched 2026-09-18

https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html fetched 2026-09-18

https://doc.rust-lang.org/stable/nightly-rustc/src/cargo/sources/git/source.rs.html fetch_db fetched 2026-09-18

https://github.com/rust-lang/cargo/blob/master/src/cargo/sources/git/utils.rs fetch and refs/commit, via local snapshot 2026-09-18

https://github.com/rust-lang/cargo/pull/11840 shallow -Zgitoxide

https://doc.rust-lang.org/cargo/commands/cargo-fetch.html

https://doc.rust-lang.org/cargo/commands/cargo-vendor.html

https://guides.rubygems.org/gemfile-lock/ fetched 2026-09-18

https://guides.rubygems.org/git/ fetched 2026-09-18

https://ruby-doc.org/3.4/stdlibs/bundler/Bundler/Source/Git/GitProxy.html fetched 2026-09-18

https://github.com/kiskolabs/pray/issues/31

usr/docs/issues/20260918204000_install-pinned-git-revision-shallow-cache.md

usr/docs/issues/20260917190500_git-cache-without-dot-git.md

usr/docs/issues/20260705204000_bundler_cargo_mix_findings.md
