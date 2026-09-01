# Yanked chacha20 0.10.1 in Cargo.lock

## Dependency

chacha20 0.10.1, pulled in by rand 0.10.2. The workspace does not declare chacha20 directly.

## Symptom

cargo publish --dry-run for pray-core warned that package chacha20 v0.10.1 in Cargo.lock is yanked on crates.io.

## Evidence

crates.io lists chacha20 0.10.1 and 0.10.0 as yanked. chacha20 0.10.2 is published and not yanked. rand 0.10.2 is the newest rand and still depends on the yanked 0.10.1 line in this lockfile.

Observed during make release-dry-run while preparing 1.9.2.

## Suggested fix

Run cargo update -p chacha20 to 0.10.2 if the rand 0.10 constraint allows it. If rand pins 0.10.1 exactly, wait for a rand release that moves the dependency, or patch. Do not add a direct chacha20 dependency only to silence the warning.

## Next

Leave the yank warning out of the 1.9.2 publish blockers. Revisit on the next lockfile refresh or if cargo deny starts rejecting yanked crates.

## Source

crates.io crate chacha20.
Cargo.lock rand 0.10.2.
usr/docs/issues/20260901170644_prepare-1-9-2-release.md
