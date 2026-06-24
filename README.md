# Prayfile

**Agent context as a dependency.**

Prayfile is an open specification for reproducible agent context composition.

It lets a project declare shared instructions, policies, memories, templates, review rituals, and workflows in one place; resolve them deterministically; lock exact versions and hashes; and render tool-specific outputs with visible provenance.

Not magic. Not agent theatre. Just dependency management for the text that bends agent behaviour.

**Status:** Draft specification v0.1 — spec-first experiment.
The open specification is the main object. The `pray` reference CLI is described in `SPEC.md`.

## Why

Agent tools now read files like `AGENTS.md`, instruction libraries, prompt templates, checklists, memories, and local workflow notes.

These files are not decoration. They shape inference. They change review behaviour. They decide what an agent notices, ignores, repeats, refuses, or breaks.

Most teams still move this context by copy-paste.

That works until it does not.

Instructions drift. Old rules stay alive. Repositories disagree. Rollbacks become folklore. Shared context turns into soft mud: almost structured, almost auditable, almost true.

Prayfile treats context like a real dependency.

Declare it. Resolve it. Lock it. Render it. Review the diff.

Every rendered fragment keeps provenance, so humans and agents can see where a piece came from instead of trusting the fog.

## What problem does it solve?

Prayfile solves reproducible composition and synchronization of shared agent context across repositories, teams, and tools.

The small problem is copying `AGENTS.md`.

The larger problem is context drift: the slow, silent divergence of rules, policies, prompts, templates, memories, and workflow assumptions.

Prayfile gives that drift a checksum.

## Is this an agentic skills framework?

No.

Skills, prompts, templates, memories, workflows, review policies, and instruction sets can be packaged with Prayfile, but Prayfile is not a runtime philosophy and not a skill altar.

Its durable concern is distribution.

It focuses on dependency management, version locking, deterministic resolution, reproducible rendering, and provenance.

## What does it do?

Prayfile:

* resolves declared context dependencies
* locks exact versions and content hashes
* produces deterministic outputs
* tracks provenance of rendered content
* keeps updates visible through ordinary version control
* avoids arbitrary package code execution

The point is not to make agents more mystical.

The point is to make context less swampy.

## Why now?

Files like `AGENTS.md` made agent context more visible and less fragmented.

That was the first step.

The next problem is distribution: how to keep shared context consistent across many repositories without turning every update into manual copy-paste jazz with broken syncopation.

Prayfile answers that part.

## Does Prayfile execute package code?

No.

Prayfile packages are data.

Resolution and rendering must be deterministic. Packages do not need to run arbitrary code to become useful.

This is a security line, not an aesthetic preference.

## Is the specification final?

No.

This is experimental.

Names, formats, package structure, resolver rules, rendering targets, and implementation details may change as the model is tested in real repositories.

The specification is the instrument. The CLI is only the first bow across the string.

## Core model

| Concept         | Role                               |
| --------------- | ---------------------------------- |
| `Prayfile`      | Human-authored dependency manifest |
| `Prayfile.lock` | Machine-authored resolved state    |
| `*.prayspec`    | Package definition                 |
| `*.praypkg`     | Package archive                    |
| `pray`          | Reference CLI                      |

## Design principles

```text
Declare context.
Resolve deterministically.
Lock exactly.
Render reproducibly.
Never execute package code.
Never hide updates.
Keep diffs small.
Preserve provenance.
```

## Repository layout

| Path             | Purpose                        |
| ---------------- | ------------------------------ |
| `SPEC.md`        | Normative specification        |
| `AGENTS.md`      | Contributor and agent workflow |
| `spec/README.md` | Test coverage guidelines       |

## Read the specification

Start with `SPEC.md`.

It defines:

* file formats
* resolver behaviour
* lockfile semantics
* package structure
* registry design
* rendering targets
* CLI commands

## Contributing

Bug reports, design critique, examples, and pull requests are welcome.

Read `CONTRIBUTING.md` before submitting changes.

The specification is currently the main area of development and feedback. Implementation should follow the spec, not outrun it into clever sludge.

## Security

Do not report security vulnerabilities through public issues.

Use the responsible disclosure process described in `SECURITY.md`.

## License

MIT. See `LICENSE.md`.
