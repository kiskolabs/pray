# Prayfile

**A package manager for agent context.**

Prayfile is an open specification for reproducible agent context composition. Declare shared instructions, policies, memories, templates, and workflows once; resolve them deterministically; lock exact versions and hashes; and render tool-specific outputs with clear provenance.

The goal is simple: treat agent context as a dependency.

**Status:** Draft specification v0.1 — **spec-first experiment**. The open specification is the primary focus. The **pray** reference CLI is described in `SPEC.md`.

## Why

Agent tools increasingly rely on files such as `AGENTS.md`, instruction libraries, prompt templates, review checklists, and other context artifacts. These files shape inference behavior, yet most teams still distribute them through copy-paste.

As context libraries grow, repositories accumulate stale instructions, hidden drift, inconsistent updates, and difficult rollbacks. Shared context becomes harder to audit and maintain.

Prayfile provides a reproducible way to package, version, distribute, and compose agent context.

Instead of manually copying files between repositories, teams declare context dependencies, resolve them deterministically, lock exact versions, and render reproducible outputs for supported tools.

Every rendered fragment retains provenance so both humans and agents can understand where shared content originated.

## FAQ

### What problem does Prayfile solve?

Reproducible composition and synchronization of shared agent context across repositories, teams, and tools.

### Is this an "agentic skills" framework?

No.

Skills, prompts, templates, memories, workflows, and instruction sets may all be packaged using Prayfile, but the durable problem is packaging and distributing context itself.

Prayfile focuses on dependency management, version locking, deterministic resolution, and provenance.

### What does the tool do?

Prayfile:

- Resolves declared context dependencies
- Locks exact versions and content hashes
- Produces deterministic outputs
- Tracks provenance of rendered content
- Enables reviewable updates through normal version control workflows

### Why now?

Broad adoption of files such as `AGENTS.md` has reduced fragmentation around how agent context is discovered and consumed.

The remaining challenge is maintaining shared context across many repositories without relying on copy-paste.

### Does Prayfile execute package code?

No.

Prayfile packages are data. Resolution and rendering are deterministic and do not require executing arbitrary package code.

### Is the specification final?

No.

The project is experimental. Terminology, formats, and implementation details may evolve as the model is validated through real-world use.

## Core Model

| Concept | Role |
|----------|----------|
| **Prayfile** | Human-authored dependency manifest |
| **Prayfile.lock** | Machine-authored resolved state |
| **\*.prayspec** | Package definition |
| **\*.praypkg** | Package archive |
| **pray** | Reference CLI |

## Design Principles

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

## Repository Layout

| Path | Purpose |
|----------|----------|
| `SPEC.md` | Normative specification |
| `AGENTS.md` | Contributor and agent workflow |
| `spec/README.md` | Test coverage guidelines |

## Read the Specification

Start with `SPEC.md` for:

- File formats
- Resolver behavior
- Lockfile semantics
- Package structure
- Registry design
- Rendering targets
- CLI commands

## Contributing

Bug reports, design discussions, and pull requests are welcome.

Please read `CONTRIBUTING.md` before submitting changes.

The specification is currently the primary area of development and feedback.

## License

MIT. See `LICENSE.md`.

## Security

Please do not disclose security vulnerabilities through public issues.

See `SECURITY.md` for responsible disclosure instructions.