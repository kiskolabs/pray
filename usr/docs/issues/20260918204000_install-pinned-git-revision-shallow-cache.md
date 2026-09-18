# Install fails when a locked git revision is missing from a shallow cache

## Participants

Vesa Vänskä
Andrei Makarov

## Decisions

Treat GitHub issue 31 as a lock-honour bug, not an update. Fetching a pinned SHA does not re-resolve versions. pray install should fetch that commit when the project cache exists and lacks it, and refuse only under --offline. No new RFC: RFC 0020 already says install fetches after using the lock, and --offline must not use the network. --locked stays a lock-change check.

Claims audit of https://github.com/kiskolabs/pray/issues/31 against this tree (pray-core current main, crates.io 1.19.0 named in the issue):

1. Existing cache plus a lock pin absent from that clone makes install fail. Outcome: supported. crates/pray-core/src/resolve_git_ensure.rs passes refresh as allow_fetch into checkout_git_revision. install sets refresh_source_revisions false.

2. Fresh cache fetches the pin. Outcome: supported. The no-.git path hardcodes allow_fetch true. Deleting the cache is the workaround that should not be required.

3. checkout_git_revision already fetches origin revision with depth 1 and blob:none, then a full fetch. Outcome: supported. crates/pray-core/src/resolve_git.rs.

4. The error names --locked though install always honours the pin. Outcome: supported. The message is in checkout_git_revision when allow_fetch is false. RFC 0020 --locked fails if the lock would change.

5. TypeScript and Ruby share the inverted fresh versus existing cache. Outcome: supported. npmjs/pray-cli/src/git/cache.ts and rubygems/pray-cli/lib/pray/git_cache.rb. Their missing-object error is a raw git checkout failure, not the --locked sentence.

6. Reporter environment pray 1.19.0, shallow cache, pin 20b4f8c3 as an ancestor of origin/main. Outcome: unverifiable in this run. The mechanism is in tree; that SHA and that machine were not replayed.

7. Suggested gate is --offline rather than refresh. Outcome: supported as the contract. RFC 0020 install --offline must not touch the network. Observation: git clone and fetch are not yet passed ResolveOptions.offline, unlike registry and tarball. This fix gates pinned checkout. A missing cache still clones during --offline. Separate follow-up.

## Effects

Engineering audit iteration 1, pipeline cache then external git then CLI error.

Finding 1. Existing shallow cache blocks honouring a lock pin. Severity: high. Confidence: high. Location: ensure_shared_git_repository and the linked worktree checkout. Kind: observed fact. Why it matters: switching branches or restoring a lock that pins an older catalog SHA fails install even though the commit is on origin. Smallest fix: allow_fetch true for a pinned revision unless offline. Pipeline: cache miss of a git object treated as a resolution refusal. Status after iteration 2: fixed in Rust, TypeScript, and Ruby.

Finding 2. Error recommends --locked. Severity: medium. Confidence: high. Location: checkout_git_revision. Kind: observed fact. Product surface: the person cannot complete install from that advice. Smallest fix: name offline when fetch is refused; drop --locked. Status after iteration 2: fixed. Offline refusal names offline mode.

Finding 3. No test for lock pin absent from a warm shallow clone. Severity: high. Confidence: high. Kind: observed fact. git_source_prepare and install_git_catalog cover unused sources, single fetch, and catalog refresh for a new package. Missing coverage, not futile coverage. Status after iteration 2: covered.

Finding 4. Git fetch on --offline is ungated for clone. Severity: medium. Confidence: high. Location: ensure_shared_git_repository clone_git_cache. Kind: observed fact. Out of scope for this pass except the pinned-checkout gate. Confirming check for a later pass: pray install --offline with no .pray/cache/git entry. Status after iteration 2: still open.

Finding 5. file:// git sources fall back to the origin worktree when cache checkout fails. Severity: medium. Confidence: high. Location: resolve_git_package_root Err arm calling local_git_source_root. Kind: observed fact, found while writing the pin-fetch fixture. HTTPS sources do not take this arm. A local file:// origin with a live distribution tree can ignore a missing pin and resolve HEAD. Out of scope for issue 31.

Boundary mode: origin is the commander for objects; pray commands fetch of the lock pin; reported HEAD can be a newer shallow tip while intended state is the lock SHA. After the fix, online install fetches the pin. Offline install alarms with an offline refusal.

Resource and budget: one depth-1 fetch of the pin, same as a fresh cache. Unmeasured CPU, disk, and energy. Confirming bench still git_source_fetch_bytes style clone_bytes before and after the pin fetch.

Trace and identification: the error prints the project cache path. Local disk path. No new telemetry. Confirming check: stderr of the offline refusal in git_source_pinned_fetch.

Skipped modes: privacy (no person data collected), observability (CLI not a long-lived service), learned systems (no model), performance numbers (no named latency budget in this pass).

Engineering audit iteration 2 after the fix.

Finding 1 closed. Online install-style resolve against a shallow cache that lacks the lock pin now returns that pin and package 1.0.0. Observed: cargo test -p pray-core --offline --test git_source_pinned_fetch, 2 passed.

Finding 2 closed. Offline error contains the pin and offline, and does not contain --locked. Same test.

Finding 3 closed. Rust resolve test, TypeScript ensureGitRepository test, Ruby GitCache.ensure_git_repository spec.

New residual after iteration 2: Finding 4 and Finding 5. Closed in the 20260918 offline clone and file-origin pass. See usr/docs/changelogs/20260918210000_offline-git-clone-and-file-origin.md.

## Next

Git-free project catalog and unshallow pin fetch shipped in usr/docs/changelogs/20260918213000_git-free-catalog-and-unshallow.md.

Ship in 1.20.0. Close GitHub issue 31 after release.

## Source

https://github.com/kiskolabs/pray/issues/31

rfcs/0020-resolve-and-lock.md sections on install, --locked, and --offline

crates/pray-core/src/resolve_git.rs

crates/pray-core/src/resolve_git_ensure.rs

crates/pray-core/src/resolve_git_source_set.rs

npmjs/pray-cli/src/git/cache.ts

rubygems/pray-cli/lib/pray/git_cache.rb

usr/docs/issues/20260908120000_install-git-catalog-refresh.md

usr/docs/changelogs/20260918204500_install-fetch-pinned-git-revision.md

usr/docs/changelogs/20260918210000_offline-git-clone-and-file-origin.md
