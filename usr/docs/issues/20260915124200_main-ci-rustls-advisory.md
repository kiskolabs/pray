# Main CI rust job fails on rustls advisory

## Participants

Andrei Makarov

## Decisions

Repair the rust GitHub Actions job that failed on main at fc8fa22 (CI run 34951572387) by moving the Cargo.lock rustls pin to the patched 0.23.45 release. Leave reqwest and the rest of the HTTP stack in place. Do not ignore RUSTSEC-2026-0285 in deny.toml. Skip CHANGELOG.md: this is a lockfile advisory bump, not a user-facing contract change.

## Effects

cargo-deny-action on main rejected rustls 0.23.43 with RUSTSEC-2026-0285. Ruby and npm jobs on the same run succeeded. Later tests, machete, coverage, and mutants on the rust job were skipped because deny failed first.

cargo update -p rustls --precise 0.23.45 locked rustls 0.23.45 and rustls-webpki 0.103.15. make audit-rust then printed advisories ok, bans ok, licenses ok, sources ok. cargo check --workspace --all-targets finished successfully on committed main sources. cargo test -p pray-core --test client_trust --test registry --test registry_search finished with 8 passed.

## Next

Open a pull request from patch/rustls-handshake-advisory with only the lockfile and these notes. Keep the local RFC 0106 / 0109 working tree off that commit.

## Source

https://github.com/kiskolabs/pray/actions/runs/34951572387
https://rustsec.org/advisories/RUSTSEC-2026-0285
usr/docs/dependencies/20260915124200_rustls-handshake-encryption-level.md
usr/docs/changelogs/20260915124200_rustls-handshake-advisory.md
