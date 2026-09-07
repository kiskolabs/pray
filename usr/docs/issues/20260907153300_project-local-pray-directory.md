# Project-local .pray directory RFC

## Participants

Andrei Makarov

## Decisions

Record the project-local .pray layout as Stable RFC 0071. Default ignore is the whole .pray directory. Hermetic work commits vendor and still ignores cache, write-state, and state.json. Write-state is Unix crash recovery and must not be committed. pray clean leaves write-state so an interrupted journal survives hygiene. RFC 0070 keeps crate map and registry cache path and points layout, ignore, state, and vendor at RFC 0071.

Do not teach pray init to rewrite gitignore in this pass.

## Effects

Claimed rfcs/ids/0071. Added rfcs/0071-project-local-pray-directory.md. RFC 0040, RFC 0070, RFC 0111, and rfcs/README.md cite it. Operator README and this repository gitignore now ignore .pray/. CHANGELOG Unreleased names the ignore guidance.

Observed: cargo test -p pray-core --test rfc_ids
5 passed, 0 failed.

## Next

Open rfc: 0071 project-local pray directory. Decide later whether init should add the ignore line, and whether Windows recovery uses the same journal.

## Source

RFC 0070 former cache-only gitignore.
RFC 0040 clean semantics.
CHANGELOG 1.12.0 Unix write recovery.
