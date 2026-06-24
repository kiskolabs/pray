# Prayfile

**A package manager for the language placed before inference.**

Prayfile is an open specification for reproducible agent context composition.

It lets projects declare shared instructions, policies, memories, templates, review checklists, and workflows in one place; resolve them deterministically; lock exact versions and hashes; preserve original source fragments; and render tool-specific outputs with clear provenance.

The goal is simple: treat agent context as a dependency.

**Status:** Draft specification v0.1 — spec-first experiment.
The open specification is the primary focus. The `pray` reference CLI is described in `SPEC.md`.

## Why

Agent tools increasingly rely on files such as `AGENTS.md`, instruction libraries, prompt templates, review checklists, memories, and workflow notes.

These files shape inference behaviour, but they are often distributed manually through copy-paste.

As context libraries grow, repositories accumulate stale instructions, hidden drift, inconsistent updates, and difficult rollbacks. Shared context becomes harder to audit, review, restore, and maintain.

Prayfile provides a reproducible way to package, version, distribute, compose, and preserve agent context.

Instead of manually copying files between repositories, teams declare context dependencies, resolve them deterministically, lock exact versions and content hashes, and render reproducible outputs for supported tools.

The lockfile records the resolved state, including checksums of the original source fragments. A local compressed cache may preserve the exact original pieces fetched from their sources. Together, the lockfile and cache make context consistency, verification, rollback, and backup possible without relying on mutable upstream state.

Every rendered fragment retains provenance so both humans and agents can understand where shared content originated.

## What problem does Prayfile solve?

Prayfile solves reproducible composition and synchronization of shared agent context across repositories, teams, and tools.

Its main concern is context drift: the gradual divergence of instructions, policies, templates, memories, and workflow assumptions between projects.

Prayfile also addresses context preservation. Resolved context should remain verifiable and recoverable even if an upstream source changes, disappears, or becomes temporarily unavailable.

## Is this an agentic skills framework?

No.

Skills, prompts, templates, memories, workflows, and instruction sets may all be packaged using Prayfile, but the durable problem is packaging and distributing context itself.

Prayfile focuses on dependency management, version locking, deterministic resolution, reproducible rendering, provenance, verification, and preservation.

## What does the tool do?

Prayfile:

* resolves declared context dependencies
* locks exact versions and content hashes
* records verifiable source checksums in `Prayfile.lock`
* may keep a local compressed cache of original source fragments
* produces deterministic outputs
* tracks provenance of rendered content
* supports consistency checks, rollback, and backup of resolved context
* enables reviewable updates through normal version control workflows
* avoids arbitrary package code execution

## Why now?

Broad adoption of files such as `AGENTS.md` has reduced fragmentation around how agent context is discovered and consumed.

The remaining challenge is maintaining shared context across many repositories without relying on copy-paste.

Prayfile addresses distribution, synchronization, locking, provenance, verification, and preservation for shared context artifacts.

## Why the name Prayfile?

The name is intentional.

`Agentfile` is too narrow. It suggests the file belongs to an agent runtime, while the specification is about context packages that may be consumed by many tools: coding agents, chat assistants, review bots, documentation generators, local inference wrappers, and future interfaces that may not call themselves agents.

`Cookfile` suggests recipes and execution. That is close to dependency composition, but misleading for this project. Prayfile does not cook, run, orchestrate, or execute package code. It resolves and renders context data.

`Mantrafile` is also close, because agent context often works through repetition, phrasing, and remembered instruction patterns. But mantra points too strongly toward repeated prompt text and too weakly toward dependency management, version locking, provenance, and recovery.

`Prayfile` describes the stranger and more accurate thing.

Modern inference systems are affected by prior text. Instructions, examples, policies, memories, formatting, headings, order, repetition, and local conventions all influence behaviour. Before the model acts, the surrounding context asks it to act in a certain way. That request may be explicit, implicit, procedural, stylistic, or structural, but it is still part of the computation.

In that sense, agent context behaves like a form of prayer: repeated language addressed to an uncertain intelligence, hoping to shape attention, judgment, refusal, style, formatting, and action.

Prayfile does not make this mystical. It makes it auditable.

If context has power, then it should have checksums. If instructions affect behaviour, then they should be versioned. If formatting changes outcomes, then rendered output should be reproducible. If teams share context, then provenance, rollback, and backup should exist.

The name keeps the philosophical point visible without changing the technical scope.

Prayfile is a package manager for the language placed before inference.

## Does Prayfile execute package code?

No.

Prayfile packages are data. Resolution and rendering are deterministic and do not require executing arbitrary package code.

This keeps package use auditable and reduces the security risks of distributing executable context tooling.

## How does Prayfile keep context consistent?

`Prayfile.lock` records the exact resolved dependency graph, selected versions, source references, content hashes, and provenance metadata.

This allows projects to verify that rendered context still matches the resolved state.

A local compressed cache may store the original source fragments used during resolution. This gives the project a recoverable copy of the resolved context inputs, independent of mutable upstream repositories, registries, or URLs.

In short:

* the lockfile verifies what was resolved
* the cache preserves what was resolved
* rendered outputs show where each fragment came from

## Is the specification final?

No.

The project is experimental. Terminology, formats, package structure, resolver rules, rendering targets, cache behaviour, registry design, and implementation details may evolve as the model is validated through real-world use.

The specification is currently the main area of development.

## Core model

| Concept         | Role                                                      |
| --------------- | --------------------------------------------------------- |
| `Prayfile`      | Human-authored dependency manifest                        |
| `Prayfile.lock` | Machine-authored resolved state                           |
| `*.prayspec`    | Package definition                                        |
| `*.praypkg`     | Package archive                                           |
| local cache     | Compressed storage for original resolved source fragments |
| `pray`          | Reference CLI                                             |

## Design principles

```text
Declare context.
Resolve deterministically.
Lock exactly.
Verify by checksum.
Cache original fragments.
Render reproducibly.
Never execute package code.
Never hide updates.
Keep diffs small.
Preserve provenance.
Support rollback.
```

## Repository layout

| Path             | Purpose                        |
| ---------------- | ------------------------------ |
| `SPEC.md`        | Normative specification        |
| `AGENTS.md`      | Contributor and agent workflow |
| `spec/README.md` | Test coverage guidelines       |

## Read the specification

Start with `SPEC.md` for:

* file formats
* resolver behaviour
* lockfile semantics
* checksum verification
* local cache behaviour
* package structure
* registry design
* rendering targets
* CLI commands

## Contributing

Bug reports, design discussions, examples, and pull requests are welcome.

Please read `CONTRIBUTING.md` before submitting changes.

The specification is currently the primary area of development and feedback.

## Security

Please do not disclose security vulnerabilities through public issues.

See `SECURITY.md` for responsible disclosure instructions.

## License

MIT. See `LICENSE.md`.
