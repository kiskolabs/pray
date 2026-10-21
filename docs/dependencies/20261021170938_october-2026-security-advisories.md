# October 2026 security advisories

## Dependency

- `rustls` 0.23.46, selected through `reqwest`
- `rustls-webpki` 0.103.16, selected through `rustls`
- `webpki-root-certs` 1.0.10, selected through `reqwest`
- `smol-toml` 1.9.1, a direct npm production dependency
- `source-map-js` 1.2.2 and `brace-expansion` 5.0.12, transitive npm development dependencies

The assessment target is the dependency graph locked on `trunk/2026w43`. The audit was triggered by the failing advisory checks described in PRs 46 and 47 and by integrating the overlapping Cargo and npm dependency branches.

## Symptom

The earlier lockfiles selected `rustls` 0.23.45, `rustls-webpki` 0.103.15, `webpki-root-certs` 1.0.8, `smol-toml` 1.8.0, and `source-map-js` 1.2.1. Published advisories identify denial-of-service and certificate-validation defects in those version ranges.

## Evidence

- RUSTSEC-2026-0346, RUSTSEC-2026-0347, RUSTSEC-2026-0348, and RUSTSEC-2026-0349 affect `rustls` before 0.23.46. Status: fixed by the selected 0.23.46 release. Applicability: the package is reachable through the HTTP client dependency graph. Priority: high because TLS is a security boundary. Disposition: accept the lockfile update and keep the advisory scan required.
- RUSTSEC-2026-0382 affects `rustls-webpki` before 0.103.16 on the selected release line. Status: fixed by 0.103.16. Applicability: reachable through `rustls`. Priority: high because the defect concerns certificate validation. Disposition: accept the lockfile update.
- RUSTSEC-2026-0383 affects `webpki-root-certs` before 1.0.10. Status: fixed by 1.0.10. Applicability: reachable through `reqwest`. Priority: high because the package supplies trust roots. Disposition: accept the lockfile update.
- GHSA-r4xh-jqrq-34v2 affects `smol-toml` through 1.8.0. Status: fixed; the lockfile selects 1.9.1. Applicability: the TypeScript CLI parses TOML with this direct production dependency. Priority: high for untrusted manifest input. Disposition: retain 1.9.1 instead of the overlapping 1.9.0 updates.
- CVE-2026-93749, also published as GHSA-68fv-2mgg-jv7q, affects `source-map-js` before 1.2.2 on its 1.x line. Status: fixed by 1.2.2. Applicability: development tooling only in this tree. Priority: medium because malicious indexed source maps can cause denial of service, but the locked package is not shipped as production CLI code. Disposition: retain 1.2.2.
- No advisory was identified for the `brace-expansion` 5.0.9 to 5.0.12 update during this assessment. Status: current lockfile passes npm audit. Applicability: development dependency. Priority: routine. Disposition: retain 5.0.12 with the integrated npm lockfile.
- `cargo deny check advisories licenses bans sources`, production and full `npm audit`, and `bundler-audit check` all completed without a vulnerability finding after integration.

## Suggested fix

Keep the integrated patched versions and their lockfiles. Do not restore the older overlapping lockfile selections. Continue running Cargo Deny, npm audit, and Bundler Audit in CI.

## Next

- Review future lockfile refreshes for regressions below these patched version floors.
- Treat new TLS or parser advisories as hot-path dependency work and repeat the target-scoped assessment.

## Source

- [PR 47](https://github.com/kiskolabs/pray/pull/47)
- [GHSA-r4xh-jqrq-34v2](https://github.com/advisories/GHSA-r4xh-jqrq-34v2)
- [GHSA-68fv-2mgg-jv7q](https://github.com/advisories/GHSA-68fv-2mgg-jv7q)
- [RUSTSEC-2026-0346](https://github.com/RustSec/advisory-db/blob/master/crates/rustls/RUSTSEC-2026-0346.md), [RUSTSEC-2026-0347](https://github.com/RustSec/advisory-db/blob/master/crates/rustls/RUSTSEC-2026-0347.md), [RUSTSEC-2026-0348](https://github.com/RustSec/advisory-db/blob/master/crates/rustls/RUSTSEC-2026-0348.md), and [RUSTSEC-2026-0349](https://github.com/RustSec/advisory-db/blob/master/crates/rustls/RUSTSEC-2026-0349.md)
- [RUSTSEC-2026-0382](https://github.com/RustSec/advisory-db/blob/master/crates/rustls-webpki/RUSTSEC-2026-0382.md) and [RUSTSEC-2026-0383](https://github.com/RustSec/advisory-db/blob/master/crates/webpki-root-certs/RUSTSEC-2026-0383.md)
- [Integration record](../issues/20261021170937_integrate-open-pull-requests.md)
