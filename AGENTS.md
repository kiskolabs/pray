# Agent context

Repository for the Agentfile open specification and the **pata** reference implementation (`pata` CLI; **seko** mix phase, **uuta** brew phase).

Read `SPEC.md` for the normative Agentfile, agentspec, lockfile, registry, and CLI design. This file defines how to work in this repository.

## Working rules

- When fixing or refactoring code, add or update tests first to expose the current bug, regression path, or missing contract; then implement the fix; then run focused and broader checks; do not ship behavior changes without proving before/after via tests.
- Test coverage must follow `spec/README.md` guidelines.
- Prefer files around 150 lines or fewer when cohesion allows; never split coherent logic purely to satisfy line count; split only when it improves ownership, readability, and reviewability.
- Use Rust and ecosystem features according to the versions declared in this repository.
- Follow Rust coding conventions, principles, and best practices.
- Do not use abbreviations or short names for variables, methods, classes, or modules unless the name is very common in the ecosystem.
- Avoid explanatory comments; allow intent comments for non-obvious constraints, invariants, concurrency edges, or external contract requirements.
- Code reflects user experience; readability, structure, and clarity are product qualities, not optional polish.
- Pull request checklist: issue ticket, changelog entry with intent or reproduction steps when relevant, test coverage, quality checks done.
- Suggest updating docs or changelog with a short summary and pull request link only when the change is significant enough to be mentioned; changelog files use `docs/changelogs/#{YYYYMMDDHHMMSS}_<title>.md`.
- Document ideas, issues, user requests, features, bugfixes, and chores in `docs/issues/#{YYYYMMDDHHMMSS}_<title>.md`.
- Branch names: `feature/<title>` for features, `patch/<title>` for bugfixes and chores, `trunk/<title>` for release-candidate or integration work before `main`, `plan/<title>` for exploration and ideation.
- Validation output must list exact commands run and observed results; never claim tests pass unless they were executed and passed.
- Ignore style-only dust unless it harms correctness, operability, maintainability, or auditability under realistic load.

## Testing

When fixing or refactoring code:

1. Add or update tests first to expose the current bug, regression path, or missing contract.
2. Implement the fix.
3. Run focused tests for the changed area, then the full test suite.
4. Do not ship behavior changes without proving before/after via tests.

Test only executable logic and user-facing behavior. Tests should affect coverage metrics.

Avoid tests that only assert implementation details. Avoid file, page, content, ordering, and regex assertions. Avoid duplicating tests.

Test coverage must follow `spec/README.md` guidelines.

Trivial one-liners need no new test.

## Rust checks

For the **pata** reference implementation, run from the workspace root:

- `cargo test` for the full suite
- `cargo test -p <crate>` for a focused crate
- `cargo clippy` and `cargo fmt --check` before claiming quality checks pass

Use coverage tooling declared in this repository when validating coverage claims.

## User-facing copy

User interface text must never mention implementation technical details.

## Pull requests

Pull request descriptions must answer:

- What problem is solved
- Why it matters
- How the solution works
- Any relevant context

For non-trivial changes, include reproduction steps or a changelog entry with intent.

## Minimal implementation

Efficient means the smallest correct change, not careless or under-tested.

Before writing code, stop at each step until one applies:

- Does the feature need to exist at all (YAGNI)?
- Does the language standard library or framework for this tree already cover it?
- Does an existing implementation or dependency already solve it?
- Can the change be one line; if so, make it one line?
- Only then write the minimum code that works.

Rules:

- Match the language of the directory you are changing.
- No abstractions unless the request or clear reuse needs them.
- No new dependency when the standard library, the framework for this tree, or an installed dependency suffices.
- No boilerplate the task did not ask for.
- Deletion over addition; boring over clever; fewest files that stay readable.
- When a request sounds overbuilt, ask whether a simpler existing path already covers it.
- When two standard-library approaches are the same size, pick the edge-case-correct one. Less code is not an excuse for a flimsier algorithm.
- Document deliberate shortcuts with an intent comment: name the known ceiling (global lock, O(n²) scan, naive heuristic) and the upgrade path when that ceiling matters.

Not optional even when minimizing scope:

- Input validation at trust boundaries
- Error handling that prevents data loss
- Security and accessibility checks where user-facing output is produced
- Calibration against real hardware and production drift when the platform ideal is not `SPEC.md`
- Anything explicitly requested in the task or ticket
- Tests for non-trivial behavior per `spec/README.md` and the testing section above

## Finite state machines

Model lifecycles with explicit finite state machines when status, allowed transitions, and side effects matter. Prefer named states and guarded transitions over scattered conditionals and implicit enums alone.

Finite state machines are not only for workflow logic. They can compactly represent ordered sets or maps of strings supporting fast prefix, suffix, and fuzzy search. Consider tries and automata when matching catalogs, codes, routes, or searchable vocabularies at scale.

## Engineering audit mode

Engineering audits and reviews use the pipeline lens and evidence-first finding format in `.agents/tech-audit/engineering-audit.md`. Read that file for dimensions, stage checks, output fields, and ranking.

Order findings by danger, then certainty, then impact, then fix cost. Smallest credible fix before structural rewrite. Separate missing coverage from futile coverage.
