# Engineering audit and 1.22.0 release prep

## Participants

Andrei Makarov

## Decisions

Run a multi-model engineering audit on the pray reference CLI after the Unreleased git-cache and publish-feedback work. Fact-check findings against source. Ship high-certainty fixes in 1.22.0. Defer larger security and resource bounds that are not introduced by this release train.

Audit agents covered git resolve and cache, CLI publish parity, test and contract readiness, and security boundary modes.

Later in the same batch, implement the deferred security and correctness items from the first audit pass, with iterative engineering audits after the fixes.

## Effects

Confirmed and fixed for 1.22.0:

- Trust verify-commit and signing metadata use the locked revision, and the gate runs before catalog unpack.
- require_signed_packages accepts only ed25519 signatures; ed25519 verify binds signer_fingerprint to signer_public_key when present.
- Path-fork refresh treats missing local content files as deletions so the merge conflict rule applies; empty-fork rematerialize runs only without a prior upstream lock.
- Broken shared bare caches rebuild via staging without rewriting the host repository origin; the cache-guard test now proves repair.
- Publish status lines redact URL userinfo; -q/-v docs and help name publish-scoped stdout; Ruby accepts -q and --quiet together; PRAY_QUIET truthy values match across Rust, Ruby, and TypeScript.
- make loc-check passes after moving global flag extraction out of main.rs.
- Ruby and TypeScript suites assert published and unchanged lines.

Deferred items implemented in this follow-up batch:

- Bound git catalog archive and blob fills; Ruby and TypeScript unpack through the same entry-type, path, and size validators as praypkg (no system tar).
- Atomic project catalog swap with mkdir-based `*.lock.d` locks around materialize (Windows-safe; no flock).
- Stub `.pray-git-dir` must resolve under the expected global bare path before trust import (lookup path).
- `record_git_revision` GIT_* isolation for path remotes under hooks; reject remotes starting with `-`.
- prayspec path validation on parse and tree hash; catalog artifact `file://` and http(s) refusal for local and remote registry paths.
- `materialize_git_catalog_file` uses `git show` only (no shared bare index mutation).
- Recovery hint regression tests in Rust, Ruby, and TypeScript, including offline wording.
- Quiet gate for apply, install, plan report, doctor, and unlock status; help and man pages name non-error status stdout.
- Doctor conformance covers layout, quiet, and URL userinfo redaction.

## Next

- After tag of the follow-up cut: crates.io, npm, RubyGems, GitHub Release, Homebrew.
- Second-iteration audit residuals (separate notes): pre-buffer size probes for git archive/show; lock stale recovery; catalog reader lock; federation/torrent absolute artifact policy; marker readers enforcing expected bare; remaining quiet writers on update/outdated listings.

## Source

Multi-model audit session 2026-10-21.

Agents: git resolve core, CLI publish parity, tests and contracts, security boundary.

Prior live-work: docs/issues/20261021194040_git-cache-legacy-and-path-fork-refresh.md

Prior live-work: docs/issues/20261021195500_publish-stdout-and-global-quiet-verbose.md
