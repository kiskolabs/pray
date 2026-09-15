# rustls handshake advisory lock bump

## Participants

Andrei Makarov

## Decisions

Move Cargo.lock rustls from 0.23.43 to 0.23.45 so cargo deny accepts the graph. rustls-webpki moved to 0.103.15 as part of that precise update. Skip CHANGELOG.md.

## Effects

Cargo.lock now pins rustls 0.23.45 and rustls-webpki 0.103.15.

Validation:

- cargo update -p rustls left 0.23.43 in place.
- cargo update -p rustls --precise 0.23.45 moved rustls 0.23.43 to 0.23.45 and rustls-webpki 0.103.13 to 0.103.15.
- make audit-rust finished with advisories, bans, licenses, and sources ok. Duplicate-version and unused-license warnings match the previous audit pass.
- cargo check --workspace --all-targets finished successfully against this lock on committed main sources.
- cargo test -p pray-core --test client_trust --test registry --test registry_search finished with 8 passed.

## Next

Merge the lock bump to main so the rust CI job can run tests again.

## Source

usr/docs/issues/20260915124200_main-ci-rustls-advisory.md
usr/docs/dependencies/20260915124200_rustls-handshake-encryption-level.md
Cargo.lock
