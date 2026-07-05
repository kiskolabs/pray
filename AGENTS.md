<!-- pray:0 ignore-comments -->

# Agent context

Do not edit managed blocks in `AGENTS.md` or skills under `.agents/`.
To change shared guidance, update `Prayfile` and run `pray install`.

## Additional instructions

### ./.agents/project.md
Repository for the pray open specification and the reference CLI.

Read `README.md` for project positioning and `SPEC.md` for the normative Prayfile, prayspec, lockfile, distribution point, and CLI design.

## Project intent

- Production readiness. Build the reference CLI and specification together, prioritizing validated contracts, user-facing reliability, and test coverage.
- Problem focus. Inference input is operational. Packaging shapes will keep changing. Prayfile targets reproducible composition, provenance markers, and sync of shared input libraries across repositories, not any one vendor workflow.
- What the tool must do. Resolve declared input dependencies, lock exact versions and hashes, render tool-specific files under defined contracts, cite managed blocks with compact pray markers into `Prayfile.lock`, and keep shared input pinned and updatable through manifest and lockfile semantics.
- Production focus. Prefer contract clarity, production validation, and test coverage over premature implementation.

## Rust workspace

For the pray reference implementation, run from the workspace root:

- `cargo test` for the full suite
- `cargo test -p <crate>` for a focused crate
- `cargo clippy` and `cargo fmt --check` before claiming quality checks pass

Use coverage tooling declared in this repository when validating coverage claims.

Prefer files around 150 lines or fewer when cohesion allows. Treat 300 lines as a hard upper bound for any source file unless a very small exception is clearly justified. When a file approaches that ceiling, split by semantic responsibility into separate modules, folders, or helpers rather than by arbitrary line count.

Test coverage must follow `spec/README.md` guidelines.

## Shared instructions

<!-- pray:7317586a -->
## Branch naming

Use kebab-case after the prefix.

Prefixes:

- `feature/<title>` — new capability
- `patch/<title>` — bugfix or chore
- `trunk/<title>` — release candidate or integration work before `main`
- `plan/<title>` — exploration or ideation

Examples:

- `feature/user-access-control`
- `patch/fix-translation`
- `trunk/2026w15`
- `trunk/2026-august-pack`
- `plan/auth-redesign-notes`
- `plan/2026-q2-roadmap`
<!-- pray:7317586a -->

<!-- pray:889f4e4f -->
---
name: changelog-update
description: Update CHANGELOG.md and docs/changelogs in amkisko house style. Use when editing changelogs, preparing releases, or syncing engineering notes into product-facing release text.
---

# Changelog update

## Two layers

1. `docs/changelogs/` — engineering draft: intent, reproduction steps, implementation notes, pull request links.
2. `CHANGELOG.md` — product-facing release notes: describe what people see and can do, not how it is built.

File name for new engineering notes: `docs/changelogs/#{YYYYMMDDHHMMSS}_<title>.md` with kebab-case title.

## Audience split

| Layer | Reader | Voice |
|-------|--------|-------|
| `CHANGELOG.md` | users, operators, product owners | outcome, screen, workflow; plain language |
| `docs/changelogs/` | engineers and reviewers | classes, files, trade-offs, links |

`CHANGELOG.md` may name an operator surface when that is the user-visible place, but still describe the workflow benefit, not internal adapter or job names.

## When to write

- user-visible features, fixes, and breaking behavior: yes;
- library upgrades, internal refactors, dev-only tooling: no unless they change a public contract or operator workflow users rely on;
- do not invent behavior; gather facts from `docs/changelogs/`, git diff, or commits since the last release tag.

## CHANGELOG.md shape

```markdown
# CHANGELOG

## X.Y.Z (YYYY-MM-DD)

- Add ...
- Fix ...
```

Rules:

- title line is `# CHANGELOG` only;
- use ISO date in parentheses on version headings when the repository follows that convention;
- unordered list with `- ` only;
- bullets stay imperative, concrete, and short;
- no marketing language;
- no negation-first hooks.

## Workflow

1. capture engineering detail in `docs/changelogs/` when the change is significant enough to mention;
2. distill user-visible outcomes into `CHANGELOG.md` when cutting a release;
3. read once for marketing odor, once for negation-led sentences, once for stray em dashes;
4. keep version headings and release tags aligned when the repository uses tagged releases.

## Relationship to pull requests

Pull request descriptions answer what problem is solved, why it matters, how the solution works, and relevant context. Changelog bullets are slightly more user-facing than commit titles but still concrete, not promotional.
<!-- pray:889f4e4f -->

<!-- pray:0b30e782 -->
## Collaboration workflow

- keep durable project context in `docs/`; use folders such as `docs/changelog`, `docs/ideas`, and `docs/tasks`;
- agent-assisted work with ongoing project value must leave a trace in the repo;
- store only specific, decision-bearing, high-signal material; do not commit generic notes, copied chat logs, or filler;
- use the lightest process that preserves traceability; design-only work does not need branch ceremony unless implementation work starts.
<!-- pray:0b30e782 -->

<!-- pray:062b8a8e -->
## Dependency issues

When work surfaces a clearly visible bug or defect in a dependency — wrong behavior, broken API contract, regression between versions, or a fix already merged upstream but not released — say so in the task output and suggest a concrete fix path: upgrade, pin, patch, vendor, workaround, or upstream report.

Store evidence under `docs/dependencies/#{YYYYMMDDHHMMSS}_<kebab-case-title>.md`; no README index in that tree. Each file should make these findable (use `##` headings or equivalent; omit empty sections): **Dependency** (name, version constraint, lockfile entry if any), **Symptom** (what breaks and where), **Evidence** (repro steps, logs, stack traces, links to issues or commits), **Suggested fix** (upgrade, pin, patch, workaround, or upstream report), **Next** (todo, planned, open questions), **Source** (links upstream—issue, PR, release note, commit—and downstream materializations in this repo). Git history is the edit log.

Do not open drive-by dependency hunts; record only issues encountered while doing the requested work and only when the defect is evident from behavior or published upstream facts, not speculation.
<!-- pray:062b8a8e -->

<!-- pray:9f724d55 -->
- docs under `docs/issues`, `docs/plan`, `docs/changelogs`, `docs/meetings`, and `docs/dependencies` use `YYYYMMDDHHMMSS_<kebab-case-title>.md`; no README index in those trees;
- any doc in those trees should make five things findable (use `##` headings or equivalent; omit empty sections): **Participants** (who was involved), **Decisions** (what was agreed), **Effects** (done, failed, recovered, rolled back), **Next** (todo, planned, open questions), **Source** (links upstream—meeting, issue, PR, commit—and downstream materializations); git history is the edit log; add an explicit note only when a later pass changes meaning (scope cut, rollback, decision reversed);
<!-- pray:9f724d55 -->

<!-- pray:c711ab37 -->
---
name: engineering-audit
description: Audit code with an evidence-first, pipeline-aware review format.
---

# Engineering audit

Use when asked for an engineering audit, systems review, hot-path analysis, Big-O review, or pipeline-style inspection.

Read `engineering-audit.md` in this skill directory for dimensions, stage checks, finding format, and ranking.

## Quick reference

Pipeline:

```text
ingress → app logic → cache → database → queue → worker → external API → egress
```

Order findings by danger, then certainty, then impact, then fix cost. Present the smallest credible fix before structural rewrite. Separate missing coverage from futile coverage.
<!-- pray:c711ab37 -->

<!-- pray:2b9051df -->
## Finite state machines

- model lifecycles with explicit finite state machines when status, allowed transitions, and side effects matter; prefer named states and guarded transitions over scattered conditionals and implicit enums alone;
- finite state machines are not only for workflow logic: they can compactly represent ordered sets or maps of strings supporting fast prefix, suffix, and fuzzy search; consider tries and automata when matching catalogs, codes, routes, or searchable vocabularies at scale.
<!-- pray:2b9051df -->

<!-- pray:b2a3d4d7 -->
## Minimal implementation

Efficient means the smallest correct change, not careless or under-tested.

Before writing code, stop at each step until one applies:
- does the feature need to exist at all (YAGNI)?
- does the language stdlib or framework for this tree already cover it?
- does an existing implementation or dependency already solve it?
- can the change be one line; if so, make it one line?
- only then write the minimum code that works.

Rules:
- match the language of the directory you are changing (see Preferred stack and tools above);
- no abstractions unless the request or clear reuse needs them;
- no new dependency when stdlib, the framework for this tree, or an installed dependency suffices;
- no boilerplate the task did not ask for;
- deletion over addition; boring over clever; fewest files that stay readable (see file size guidance above);
- when a request sounds overbuilt, ask whether a simpler existing path already covers it;
- when two stdlib approaches are the same size, pick the edge-case-correct one; less code is not an excuse for a flimsier algorithm;
- document deliberate shortcuts with an intent comment: name the known ceiling (global lock, O(n²) scan, naive heuristic) and the upgrade path when that ceiling matters.

Not optional even when minimizing scope:
- input validation at trust boundaries;
- error handling that prevents data loss;
- security and accessibility (see UI/UX checks);
- calibration against real hardware and production drift when the platform ideal is not the spec;
- anything explicitly requested in the task or ticket;
- tests for non-trivial behavior per @spec/README.md and the testing bullets above; trivial one-liners need no new spec.
<!-- pray:b2a3d4d7 -->

<!-- pray:6aea78d0 -->
## Preferred stack and tools

- native-first approach for all platforms and languages
- ruby for web application and API development, and for its rich ecosystem of libraries and frameworks
- elixir for concurrent and distributed systems, and for its actor model and fault tolerance
- rust for system programming and performance-critical code
- javascript, html, css for native browser experience
- humane and accessible design principles for UI/UX, and for clear communication of intent and feedback
<!-- pray:6aea78d0 -->

<!-- pray:e662c764 -->
## Checks before publish (engineering)

Verify the change is wanted, discuss first for unconfirmed larger features, describe what problem is solved and why it matters, include tests, add screenshots or screen recordings for UI changes, keep one PR to one concern, and understand any AI-assisted code you submit.
<!-- pray:e662c764 -->

<!-- pray:8cf2baf2 -->
## Likely rejected changes

- features whose complexity outweighs user value
- giant refactors
- non-trivial changes without tests
- style-only rewrites without behavior change
- AI-generated-looking code the author does not understand
<!-- pray:8cf2baf2 -->

<!-- pray:7de8c0b2 -->
- use Rust and Cargo features according to the versions declared in the repository;
- follow Rust API guidelines, idiomatic error handling (`Result`/`Option`), and clippy-backed conventions where the project enables them;
- prefer explicit crate boundaries; keep binaries thin and library code testable;
- test coverage must follow the conventions declared in the relevant subtree; when a project defines coverage rules in `spec/README.md` or equivalent, follow those;
<!-- pray:7de8c0b2 -->

<!-- pray:5ef025d3 -->
- when fixing or refactoring code, add or update tests first to expose the current bug/regression path (or missing contract), then implement the fix, then run focused and broader checks, and do not ship behavior changes without proving before/after via specs;
- test only executable logic and user-facing behavior; tests should affect coverage metrics;
- avoid tests that only assert implementation details; avoid file/page content/ordering/regex assertions; avoid duplicating tests;
- user interface texts should never mention implementation technical details;
- prefer files around <=150 LOC when cohesion allows, but never split coherent logic purely to satisfy line count; split only when it improves ownership, readability, and reviewability;
- do not use abbreviations and short names for variables, methods, classes, etc. unless it is a very common abbreviation or short name;
- avoid explanatory comments, but allow intent comments for non-obvious constraints, invariants, concurrency edges, or external contract requirements;
- keep the idea that code reflects user experience, so readability, structure, and clarity are product qualities, not optional polish;
- pull request description should include answers to questions: what problem is solved, why it matters, how the solution works, and any relevant context; if the change is non-trivial, include reproduction steps or a changelog entry with intent;
- pull request checklist: changelog entry with intent or reproduction steps when relevant, test coverage, and quality checks done;
- suggest updating docs/changelog with a short summary and PR link only when the change is significant enough to be mentioned; changelog files should use `docs/changelogs/#{date +"%Y%m%d%H%M%S"}_<title>.md`;
- when documenting ideas, issues, user requests, new features, bugfixes, chores, etc., use `docs/issues/#{date +"%Y%m%d%H%M%S"}_<title>.md`;
- validation output must list exact commands run and observed results, and never claim tests pass unless they were executed and passed;
- ignore style-only dust unless it harms correctness, operability, maintainability, or auditability under realistic load.
<!-- pray:5ef025d3 -->

<!-- pray:c7597e52 -->
## Writing and changelog prose checks

Read once for marketing odor, once for negation-led sentences, once for stray em dashes, and once for paragraphs that break on clause instead of on scene; keep live notes and metadata honest and plain.
- repo docs under docs/issues, docs/tasks, and docs/changelogs: plain prose readable without a rendered preview—no markdown tables, bold, italic, or other styling; prioritize factual accuracy over presentation.
<!-- pray:c7597e52 -->

<!-- pray:c41cee92 -->
---
name: prayer-publisher
description: Turn source text, files, or folders into packaged prayer and publish it.
---

# Prayer Publisher

## Purpose

Turn a source text file, folder, or existing skill into a packaged prayer and publish it through the Pray workflow.

## When to use

Use this skill when you want to:

- package existing guidance as prayer-managed input
- convert one file or many files into a skill directory
- install the result under `.agents/skills`
- build a `.praypkg` and publish it to a distribution point

## Inputs

Preferred inputs:

- a single Markdown or text file
- a folder of Markdown/text files
- an existing skill directory with `SKILL.md`
- the target skill name
- the desired package name and version

## Process

1. Identify the source root.
   - If the input is a folder, decide which files belong in the package.
   - Keep the source static and deterministic.

2. Shape the package layout.
   - Put the main skill entrypoint at `skills/<name>/SKILL.md`.
   - Put support files under `assets/`, `examples/`, or `templates/`.
   - Keep scripts inert if present.

3. Declare every file explicitly.
   - List each packaged file in `spec.files`.
   - Do not rely on hidden globbing or implicit discovery.
   - Preserve the package tree shape exactly.

4. Add the consumer `Prayfile` entry.
   - Declare the package source.
   - Opt into the skill export.
   - Render to `.agents/skills` or the target skills directory.

5. Resolve and render.
   - Resolve the package deterministically.
   - Render the skill into the target repository.
   - Keep churn minimal and output stable.

6. Verify the result.
   - Run `pray verify` to check checksums, marker positions, and lockfile consistency.
   - Run `pray drift` when you want a broader drift report.

7. Package and publish.
   - Build the package with `pray package`.
   - Publish it with `pray publish` to a configured distribution point.

## Output

A successful run produces:

- a normalized skill directory
- a `*.prayspec` that declares the skill files and exports
- a `Prayfile` that opts the skill into a target
- a rendered copy under `.agents/skills/<name>`
- a locked, verifiable state in `Prayfile.lock`
- a publishable `.praypkg`

## Practices

- Prefer small, reviewable diffs.
- Keep `SKILL.md` as the canonical entrypoint.
- Keep provenance in `Prayfile.lock`, not in rendered files.
- Do not execute package code.
- Prefer explicit paths over magic.
- Treat manual edits to managed output as drift unless the workflow says otherwise.

## Good defaults

- Name the skill after the behavior it provides.
- Keep supporting files close to `SKILL.md`.
- Include examples only when they help use or review the skill.
- Use `render mode: :managed` for repository-shared output.
- Use `conflict: :fail` unless there is a strong reason to do otherwise.
<!-- pray:c41cee92 -->
