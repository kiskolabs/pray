# h2 empty data frame advisory

## Dependency

`h2` was locked at 0.4.15 through the `reqwest` and `hyper` HTTP graph. The workspace does not declare `h2` directly.

## Symptom

The advisory scan rejected the lockfile because h2 0.4.15 could permit unbounded processing of empty HTTP/2 DATA frames.

## Evidence

`cargo deny check advisories licenses bans sources` reported RUSTSEC-2026-0258 against h2 0.4.15 and identified 0.4.16 as the patched release.

After the lockfile update, the same command completed with advisories, bans, licenses, and sources all accepted.

## Suggested fix

Keep h2 at 0.4.16 or later within the existing compatible dependency constraints. No source patch or direct h2 dependency is needed.

## Source

RustSec advisory: https://rustsec.org/advisories/RUSTSEC-2026-0258.html

Downstream lockfile: `Cargo.lock`.
