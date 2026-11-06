# Rust cargo upgrade QA on trunk/2026w45

## Decisions

Run full cargo update on trunk/2026w45 after Dependabot merges and parent npm/Ruby upgrades.

Bump only the toml floor in crates/pray-core/Cargo.toml from 1.1 to 1.2 so the lock resolves to crates.io latest (1.2.0+spec-1.1.0). Leave other direct floors as major/minor ranges; they already resolve to latest compatible crates.io versions. Do not tighten patch floors.

Leave worktree cargo changes uncommitted for the parent agent to consolidate. Do not push.

## Effects

Worktree created from trunk/2026w45 at 3a00d807ad70ab8fffc71e38d04f0c61456a46da. Shared WORKTREE_ID=cargo-upgrade-af6e517e. Setup skipped after checking REPO_ROOT and WORKTREE_PATH for .cursor/worktrees.json (neither present).

cargo update locked 0 additional packages beyond the parent-synced lockfile. 116 packages changed versus the branch HEAD lock; notable direct-related locks include toml 1.1.8+spec-1.1.0 to 1.2.0+spec-1.1.0, and (versus origin/main) rusqlite 0.40.2 to 0.41.1 and jsonschema 0.58.6 to 0.60.2 from earlier Dependabot merges.

QA commands and results:

- cargo fmt --check: PASS
- cargo clippy --workspace --all-targets --all-features -- -D warnings: PASS
- cargo test --workspace: FAIL without PRAY_TRUST_ASSUME_YES (cli_upgrade::upgrade_notice_uses_changelog_link needs interactive trust or trusted signer in non-TTY). PASS with PRAY_TRUST_ASSUME_YES=1
- make loc-check: PASS
- make coverage-rust: PASS (TOTAL lines 76.52%)
- make audit-rust: PASS (advisories ok, bans ok, licenses ok, sources ok)
- make libyears-rust: PASS (system 39.65 libyears behind; top: anes 5.59, itertools 2.08, bit-vec 1.98, bit-set 1.98, windows-sys 1.89)
- make size-check: FAIL overall (npm build missing tsc in worktree). Rust binary alone ok: 8976432 bytes under 9500000 budget
- make bench: FAIL at pray-bench operations orphan_marker_scan assertion (left 11 right 10 managed span count). Install pipeline benches completed before the assertion

Upgrade-resource compare versus origin/main release pray --help (3 samples, median):

- Cargo.lock bytes: base 73904, head 73409, delta -495
- release pray bytes: base 8927920, head 8976432, delta +48512
- target dir bytes after release build -p pray-cli: base 368988160, head 364179456, delta -4808704
- RSS median bytes: base 3293184, head 3276800, delta -16384
- CPU median seconds per --help (200-loop wall / loops): base about 0.003636, head about 0.003820, delta about +0.000184

Direct third-party floors checked against crates.io max_stable: all at latest within declared ranges. No further safe floor bumps taken beyond toml 1.2. Transitive libyear lag remains (anes, itertools, windows-* family, and similar).

## Next

Parent consolidates Cargo.lock and crates/pray-core/Cargo.toml via /apply-worktree or copy from this worktree.

Investigate bench fixture managed span count mismatch (11 vs 10) separately if it reproduces on a clean TTY checkout without PRAY_TRUST_ASSUME_YES side effects.

Install npm toolchain in worktrees when size-check must cover npm packs, or run size-check only for the rust_binary line when npm is out of scope.

Optional: decide whether cli_upgrade trust consent should use an env escape in CI/agent non-TTY without relying on operator trust keys.

## Source

Worktree path under ~/.cursor/worktrees/cargo-upgrade-af6e517e/. Branch tip trunk/2026w45. Base for resource compare: origin/main 46a2427. Head cargo state: uncommitted Cargo.lock plus toml floor bump on top of 3a00d80. Related prior Dependabot merges already on the trunk branch for rusqlite and jsonschema.
