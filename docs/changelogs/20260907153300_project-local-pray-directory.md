# Project-local .pray directory RFC

## Participants

Andrei Makarov

## Decisions

RFC 0071 is the contract for .pray children, default ignore, hermetic vendor exception, and write-state as ignored recovery. RFC 0070 drops duplicate layout text. RFC 0040 clean leaves write-state.

## Effects

New Stable Standards Track RFC 0071. Operator README recommends .gitignore .pray/. This repository gitignore now covers write-state.

Observed: cargo test -p pray-core --test rfc_ids
5 passed, 0 failed.

## Next

Merge after rfc_ids and review. Init gitignore write and Windows journal parity stay unresolved in the RFC.

## Source

rfcs/0071-project-local-pray-directory.md
usr/docs/issues/20260907153300_project-local-pray-directory.md
CHANGELOG.md Unreleased
