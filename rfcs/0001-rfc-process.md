# RFC 0001: RFC process

- Feature Name: rfc-process
- Type: Procedural
- Status: Stable
- Created: 2026-08-17
- Updated: 2026-08-18
- Author: Andrei Makarov
- Stakeholders: project maintainers
- Relates: RFC 0111

## Summary

This RFC defines how Prayfile specification and implementation changes are proposed, written, numbered, and advanced. Numbered RFCs in this directory are the product contract.

## Motivation

A single specification snapshot grew to 81 top-level sections and about 2700 lines. Implementation lives in `pray-core`, `pray-cli`, `pray-transport`, `pray-bench`, and the Ruby and TypeScript CLIs. Reviewers need a unit smaller than that file and larger than a pull request description.

## Guide-level explanation

Claim `ids/NNNN` with one line: the kebab slug, or `reserved` then the slug. Copy `0000-template.md`, delete the optional-header instruction, fill the sections that apply, omit unused header fields and empty sections, and open `rfc: NNNN short title`. Two pull requests that add the same `ids/NNNN` path conflict in git. Duplicate `NNNN-*.md` files fail `cargo test -p pray-core --test rfc_ids`. Discussion is the pull request. Stakeholders are maintainers plus owners of the touched area. The default clock is two weeks of lazy consensus.

After merge, implementation PRs cite the RFC number. Contract text lives in the RFC. Changelogs record versions.

An RFC proposes a design: suggestion, motivation, specification, effects, alternatives, and prior art. Version numbers belong in changelogs. Implementation notes stay optional evidence. Shipped behavior is already accepted, so an RFC that specifies already-shipped design is Stable on merge. Experimental means the design is not yet the product contract. Proposed means a change is under review. The two-week clock applies to Proposed changes.

Trivial exemption: a bugfix that restores documented behavior, a typo, or a refactor that does not change bytes a user can observe.

## Reference-level explanation

### Isolation

RFCs MUST NOT cite markdown files outside this directory. Restate the fact, or cite another RFC. Implementation notes MAY name crates, modules, schemas, and fixtures.

### Header

Required: Type, Status, Created, Author. The title is the H1.

Optional, omit unused: Feature Name, Describes, Stakeholders, Feedback until, Relates, Requires, Supersedes.

Describes is optional and historical. Existing Informational RFCs 0002–0070 and 0101 use `Describes: 1.8.1`. New RFCs omit Describes. The RFC subject is the design. Stakeholders and Feedback until belong on Proposed RFCs.

### Shape

Required sections: Summary, Motivation, Guide-level explanation, Unresolved questions. Summary is one paragraph: the suggestion. Product RFCs also fill Reference-level explanation (the specification), Drawbacks (effects), Rationale and alternatives, and Prior art. Implementation notes are optional. Omit unused sections.

### Length

Prefer staying well under 300 lines. Approaching 300 lines is the cognitive red zone for human reading. A coherent contract MAY continue past 300. Hard maximum is 1000 lines. Split a second RFC when a file grows because it has two concerns, especially as it nears 300. Do not pad.

Source-file line limits for Rust, Ruby, and TypeScript (warn at 150, fail above 300) are a separate rule in RFC 0070. They do not apply to RFC prose.

### Prose

State the fact, delete fence tags (`not X`, `instead of Y`) that only block a misreading, and open with the claim. Keep agency on the person who acts; tools do mechanical work. Prefer commas and full stops over em dashes. Refuse sales language, methodology pitch, landing-copy slogans, hero close, punchline stacks of one-clause sentences, and outline cadence dressed as prose.

Safety and contract negation stay: `MUST NOT execute package code` is a rule.

### Claims

A checkable statement MUST name a command, field, fail mode, fixture, schema, crate, or test a reviewer can open. Mark inference. Implementation paths are optional evidence. Analogies (Bundler, XEP) belong under Prior art.

### Types, statuses, running code

Types and statuses are listed in `rfcs/README.md`. Humorous documents are out of scope. Standards Track RFCs SHOULD show running code or fixtures before Stable. An RFC that specifies already-shipped design MUST be Stable; writing the RFC later does not reopen the feature. Prototype code for a contract that is not yet the product MAY remain Experimental.

### Number assignment

The author claims a number from the `rfcs/README.md` bands by adding `ids/NNNN`. The claim file is the reservation. Two in-flight pull requests that pick the same number both add that path, so git shows an add/add conflict. After a conflict, the later change takes a free id and updates the draft filename. A second `NNNN-*.md` without a matching claim fails the `rfc_ids` test.

## Implementation notes

Id claims are checked by `crates/pray-core/tests/rfc_ids`. Other repositories that copied this process wait for Stable before extracting a prayer; they do not fork the process text while this RFC is Proposed.

## Drawbacks

Authors pay process overhead. The trivial exemption and Stable status for already-shipped design keep that cost down. Large format RFCs (Prayfile statements, lockfile example) sit in the red zone by design; splitting them would scatter one grammar.

## Rationale and alternatives

A single snapshot failed as a review unit once it passed a few thousand lines. Engineering traces record decisions. Conformance lives in RFCs and fixtures. Importing the Rust RFC template unchanged would drop type, status, and registrar. Importing XEPs unchanged would invent Council and Board roles this project does not have.

## Prior art

Rust RFC template; Mozilla Android RFC stakeholders and feedback window; XEP-0001 types and Experimental to Final; W3C RFC 2119 keywords on Standards Track text. Prose adapted from amaaov.github.io text rules, excluding blog-only literary, scene, translation, and visual rules.

## Unresolved questions

Whether polyglot CLIs need a separate note when only one runtime implements a surface.
