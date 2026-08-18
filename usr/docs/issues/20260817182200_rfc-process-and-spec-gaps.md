# Spec and implementation gaps found while starting RFCs

## Participants

Andrei Makarov

## Decisions

Gaps found while mapping SPEC.md to crates and polyglot CLIs are recorded as Experimental RFCs rather than silent SPEC edits. Reserved numbers hold work that is real but not drafted in this pass.

## Effects

RFC 0100: fixtures/ and testdata/shared are too small for three CLIs to claim shared conformance. SPEC.md does not use RFC 2119 keywords.

RFC 0101: SPEC.md section 75 names crates that do not exist. pray-core is the library; pray-transport overlaps registry modules in core.

RFC 0102: destination DSL is recommended and warned, but SPEC.md still leads with target, output, and agent. Render conflict policies other than fail are specified and rejected by the parser.

RFC 0104: federation and extra transports have code and long issues, without a Stable wire contract. Level 3 should stay path, tarball, git, and static HTTP.

RFC 0111: the specification snapshot is 81 top-level sections plus 32.1 and 32.2, 2699 lines, mixing vision, algorithms, milestones, and aspirational architecture.

Also noted, reserved not drafted:

RFC 0103 environment and group render selectors already shipped.

RFC 0105 trust enrollment completeness versus CLI login, tokens, and passkey flags.

RFC 0106 host-language adapter that reads Prayfile.lock without replacing the CLI.

RFC 0107 search must stay unranked; storefront ranking remains a non-goal.

RFC 0110 marker id stability and preamble drift after the 1.8.1 grouping fix.

RFC 0112 help, man page, and exit-code conformance versus SPEC.md section 50 lagging the Command enum (unlock, login, token, search, yank, trust, sync, completion, upgrade).

RFC 0113 independent parsers versus wrapper CLIs.

Production-readiness still wants a two-machine network path with injected failure. Clippy has no file length lint yet.

## Next

Prioritize RFC 0100 fixtures and RFC 0102 SPEC example order. Fold environment into 0020/0030 or draft 0103. Do not split crates until RFC 0101 recommends a consumer.

Decide whether require-signed should accept only ed25519 signatures. Decide whether install HTTP should call HttpTransport or stay in pray-core. Two-machine injected-failure fixture still missing.

## Source

Upstream: SPEC.md, crates/pray-core, crates/pray-cli/src/command.rs, crates/pray-transport, schema/, fixtures/, testdata/shared, docs/cli-exit-codes.md, usr/docs/issues on federation, transport, trust, zero-trust render, and production readiness.

Downstream: rfcs/0100-conformance.md, rfcs/0101-crate-boundaries.md, rfcs/0102-destination-dsl.md, rfcs/0104-federation-transports.md, rfcs/0111-spec-as-rfc-index.md, rfcs/README.md reserved list.
