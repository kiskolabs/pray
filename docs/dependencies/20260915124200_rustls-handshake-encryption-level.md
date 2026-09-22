# rustls handshake encryption level advisory

## Dependency

rustls was locked at 0.23.43 through reqwest 0.13.4, hyper-rustls 0.27.9, tokio-rustls 0.26.4, rustls-platform-verifier 0.7.0, and jsonschema 0.53.0. The workspace does not declare rustls directly.

## Symptom

GitHub Actions rust job on main failed at cargo-deny-action. cargo deny reported RUSTSEC-2026-0285 against rustls 0.23.43: TLS 1.3 handshake messages accepted across encryption level boundaries.

## Evidence

CI run 34951572387 on fc8fa22 failed the rust job at EmbarkStudios/cargo-deny-action. Ruby and npm jobs on that run succeeded. Solution text from cargo deny: upgrade to >=0.23.45.

RUSTSEC-2026-0285 / GHSA-2mjx-qc3c-rqvc, issued 2026-09-14. Affected rustls versions are 0.23.13 through 0.23.44. Patched release is 0.23.45. CVSS 3.1 5.3 MEDIUM (AV:N/AC:L/PR:N/UI:N/S:U/C:L/I:N/A:N). crates.io lists rustls 0.23.45 as max_stable, published 2026-09-14T15:11:17Z.

cargo update -p rustls left rustls at 0.23.43. cargo update -p rustls --precise 0.23.45 moved rustls to 0.23.45 and rustls-webpki to 0.103.15.

After that lock, make audit-rust printed advisories ok, bans ok, licenses ok, sources ok. cargo check --workspace --all-targets finished successfully. cargo test -p pray-core --test client_trust --test registry --test registry_search finished with 8 passed.

## Suggested fix

Keep rustls at 0.23.45 or later within the existing ^0.23 constraints. No source patch or direct rustls dependency is needed.

## Trigger

cargo-deny-action on main CI, RUSTSEC-2026-0285 against Cargo.lock rustls 0.23.43.

## Assessment target

kiskolabs/pray Cargo.lock HTTP and TLS graph used by pray-cli, pray-core, and pray-transport at version 1.15.0. Runtime path is reqwest with the rustls feature. jsonschema pulls rustls on the pray-core dev graph.

## Advisory

RUSTSEC-2026-0285, GHSA-2mjx-qc3c-rqvc. Same class as Go GO-2026-4340 / CVE-2025-61730.

## Status

fixed in this working tree after the lock bump. main at fc8fa22 remains affected until this lock ships.

## Applicability

rustls 0.23.43 was present and on the TLS 1.3 handshake path for HTTPS registry and distribution traffic. The published issue is acceptance of a handshake message at the wrong encryption level when it follows a key-changing message in the same record. The RustSec text says the handshake transcript stays authenticated, so a network-position attacker cannot use this to alter or complete a handshake. The GHSA impact line says an on-path attacker can inject plaintext messages that are accepted. That is enough to treat the 0.23.43 lock as affected for this target.

## Priority

CI on main is red. The crate is transitive on the HTTP hot path. A patched crates.io release exists. CVSS confidentiality impact is low; integrity and availability are none.

## Disposition

Upgrade the lockfile. Do not ignore the advisory in deny.toml. Do not add a second TLS stack.

## Source

https://rustsec.org/advisories/RUSTSEC-2026-0285
https://github.com/rustls/rustls/security/advisories/GHSA-2mjx-qc3c-rqvc
https://github.com/kiskolabs/pray/actions/runs/34951572387
Cargo.lock
usr/docs/issues/20260915124200_main-ci-rustls-advisory.md
