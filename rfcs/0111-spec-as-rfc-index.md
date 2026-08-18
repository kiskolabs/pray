# RFC 0111: RFCs as the specification

- Feature Name: spec-as-rfc-index
- Type: Procedural
- Status: Stable
- Created: 2026-08-17
- Updated: 2026-08-18
- Author: Andrei Makarov
- Relates: RFC 0001, RFC 0100
- Requires: RFC 0001
- Supersedes: the former snapshot-as-canonical design of this RFC

## Summary

Numbered RFCs are the Prayfile product contract. The former specification snapshot is retired. Readers start from `rfcs/README.md` and open the RFC that owns the concern.

## Motivation

Keeping a compiled snapshot beside RFCs duplicated MUST text and hid which chapters were aspirational. Reviewers already work in RFC units. One extra 2700-line file added search cost without adding a second source of truth.

## Guide-level explanation

Open `rfcs/README.md` for types, statuses, numbering, and the current set. Open the RFC that owns the concern:

- RFC 0002: problem and positioning
- RFC 0010: Prayfile surface
- RFC 0011: prayspec and package archive
- RFC 0020: lock and resolve
- RFC 0030: render, markers, verify, drift
- RFC 0031: ownership zones
- RFC 0040: CLI verbs, config, environment, exit codes
- RFC 0050: security and trust
- RFC 0060: static registry and sources
- RFC 0070: reference implementation and operator layout
- RFC 0100: conformance fixtures
- RFC 0101: crate boundaries
- RFC 0102: destination DSL as canonical examples
- RFC 0104: federation and extra transports
- RFC 0108: file-as-fragment

Implementation PRs cite `RFC-NNNN`. JSON Schema and fixtures win for field presence. RFC reference-level text wins for algorithms until RFC 0100 is Stable.

## Reference-level explanation

RFCs in this directory are the specification. Operator docs MAY cite an RFC number. They MUST NOT treat a retired snapshot path as normative.

Aspirational text MUST live in Experimental RFCs. Normative algorithm text SHOULD use RFC 2119 keywords (RFC 0100).

Length follows RFC 0001: red zone near 300 lines, hard maximum 1000.

## Drawbacks

Readers who wanted one file now follow an index. Mitigate with the mapping in this RFC and `rfcs/README.md`.

## Rationale and alternatives

Keep a hand-edited snapshot: duplicates the contract and stalled review. Auto-generate a snapshot from RFCs: prose quality would drop, and the generator does not exist. RFCs as the only contract matches how changes are already proposed.

## Prior art

Rust RFCs plus a language reference is the rejected alternative. W3C publishes per-TR contracts rather than one mega-snapshot.

## Unresolved questions

Whether JSON Schema or an RFC wins when they disagree on field presence. Recommendation: schema plus fixtures win for field presence; RFC text wins for algorithms until RFC 0100 is Stable.
