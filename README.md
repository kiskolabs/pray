# Agentfile

Open, lockfile-based package specification for reusable AI-agent context.

Agentfile lets repositories declare shared instructions, skills, templates, and tool-specific context packages once, resolve them reproducibly, and render small deterministic outputs for different agent tools.

**Status:** Draft specification v0.1. The **pata** reference CLI (**seko** mix phase, **uuta** brew phase) is in progress in this repository.

## Why

Agent-oriented development relies on files such as `INSTRUCTIONS.md`, `.tool-c/rules/`, skills, and review checklists. Teams copy the same context between repositories, which leads to stale instructions, large diffs, hidden drift, and hard rollback.

Agentfile treats agent context as a dependency—similar in spirit to a dependency manifest and lockfile, but without requiring host-language execution.

## Core model

| Concept | Role |
|---------|------|
| **Agentfile** | Human-authored dependency manifest |
| **Agentfile.lock** | Machine-authored exact resolved state |
| **\*.agentspec** | Package definition file |
| **\*.agentpkg** | Built package archive |
| **pata** | Reference CLI |
| **seko** | Mix phase — resolve dependencies and merge exports |
| **uuta** | Brew phase — fetch, verify, and render context |

```
Parse data.
Resolve deterministically.
Lock exactly.
Render minimally.
Never execute package code.
Never hide updates.
Keep diffs small.
```

## Repository layout

| Path | Purpose |
|------|---------|
| [SPEC.md](SPEC.md) | Normative open specification |
| [AGENTS.md](AGENTS.md) | Contributor and agent workflow for this repo |
| [spec/README.md](spec/README.md) | Test coverage guidelines |

## Read the specification

Start with [SPEC.md](SPEC.md) for file formats, resolver rules, lockfile semantics, registry design, rendering targets, and CLI commands.

## Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/kiskolabs/agentfile.

See [CONTRIBUTING.md](CONTRIBUTING.md) for the golden rule of automation, development setup, and checks.

## License

MIT. See [LICENSE.md](LICENSE.md).

## Security

Do **not** open a public issue for security vulnerabilities. See [SECURITY.md](SECURITY.md).
