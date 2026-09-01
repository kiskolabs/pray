# Pray RFCs

RFCs are the Prayfile specification and the unit of design review for `pray` implementation changes. An RFC proposes a design: suggestion, motivation, specification, effects, alternatives, and prior art. Version numbers belong in changelogs.

RFCs MUST NOT cite markdown files outside this directory. Cross-references stay among RFCs (`RFC 0001`, `0000-template.md`, this `README.md`). Implementation notes MAY name crates, modules, schemas, and fixtures.

Shape follows the Rust RFC template: Summary, Motivation, Guide-level explanation, Reference-level explanation, Drawbacks, Rationale and alternatives, Prior art, Unresolved questions, Future possibilities. Type, status, registrar, and running-code rules come from XEP-0001. Stakeholders and a feedback window come from Mozilla Android RFCs. Omit empty sections.

Writing, length, and claim rules live in RFC 0001. RFC 0111 retired the specification snapshot.

## Types

- Standards Track: a wire, file, or CLI contract implementations MUST follow
- Informational: description or analysis that does not by itself change a contract RFC
- Historical: a shape that shipped before this process
- Procedural: how this project decides, numbers, and advances RFCs

## Statuses

- Draft: authoring; not yet listed as published
- Experimental: published design that is not yet the product contract
- Proposed: a change open for review toward Stable
- Stable: accepted. Behavior that already shipped is Stable when the RFC specifies it; writing the RFC later does not reopen the feature
- Final: deployed long enough that breaking changes need a new RFC
- Deferred, Rejected, Superseded, Obsolete

The two-week lazy-consensus clock applies to Proposed changes.

## Numbering

- 0000: template
- 0001-0009: process and positioning
- 0010-0019: core formats
- 0020-0029: resolve and lock
- 0030-0039: render, markers, verify, ownership
- 0040-0049: CLI
- 0050-0059: security and trust
- 0060-0069: distribution
- 0070-0079: reference implementation
- 0100+: follow-on contracts

Claim an unused id in `ids/NNNN` before writing `NNNN-slug.md`. The file holds one line: the slug, or `reserved` then the slug. Two pull requests that add the same `ids/NNNN` path conflict in git. Duplicate drafts fail `cargo test -p pray-core --test rfc_ids`.

## Lifecycle

1. Claim `ids/NNNN`.
2. Copy `0000-template.md`. Omit unused header fields and empty sections.
3. Open `rfc: NNNN short title` from `plan/` or `feature/`.
4. Discuss until Summary, Motivation, and Unresolved questions are honest.
5. If the RFC specifies already-shipped design, mark Stable in the same PR. Otherwise mark Experimental, Proposed, Stable, Rejected, or Deferred.
6. Implementation PRs cite `RFC-NNNN`. Contract edits land in the RFC. Informational as-built RFCs do not wait for a second implementation PR.

Trivial exemption: bugfixes, typos, and refactors that do not change user-facing contracts.

## Current set

Procedural: RFC 0001 (Stable), RFC 0111 (Stable).

Informational, Describes 1.8.1, Stable: RFC 0002, 0070, 0101.

Standards Track, shipped design, Stable: RFC 0010, 0011, 0020, 0030, 0031, 0040, 0050, 0060.

Standards Track follow-ons (not yet the product contract): RFC 0100, 0102, 0104, 0108.

Standards Track Proposed: RFC 0051 (registry authentication delivery and enrollment).

Reserved (see `ids/`): 0103 lockfile environment (optional field already in `lockfile.schema.json`; an RFC would canonize it); 0105 trust enrollment; 0106 host-language lock adapter; 0107 search ranking; 0110 marker-id stability; 0112 help/man/exit codes; 0113 independent parsers.
