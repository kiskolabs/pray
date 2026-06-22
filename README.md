# Agentfile

**A lockfile-resolver for agent context text files.**

Open specification for reproducible context alteration: declare shared instructions and recipes once, resolve them deterministically, render tool-specific files, and leave provenance markers showing where each piece came from.

**Status:** Draft specification v0.1 — **spec-first experiment**. The open specification is the current focus. The **pata** reference CLI (**seko** mix phase, **uuta** brew phase) is designed in [SPEC.md](SPEC.md).

## Why

Agent tools alter context by reading files such as `AGENTS.md`, `.agents/`, instruction files, templates, and review checklists. Context alteration is core to inference; the packaging shape around it will keep changing.

There is no bundler for these text files today. Teams copy the same context between repositories, which leads to stale instructions, large diffs, hidden drift, and hard rollback.

Agentfile treats agent context as a dependency—similar in spirit to a dependency manifest and lockfile, but without requiring host-language execution. It alters files under defined contracts and records the source of each alteration.

## FAQ

**What problem is this actually solving?**  
Reproducible context alteration and sync of shared libraries/recipes across repositories—not a bet on any single agent workflow or packaging fad.

**Is this an "agentic skills" product?**  
No. Skills may be one artifact type a target renders, but the durable problem is distributing, locking, and rendering **text context** with reviewable diffs and clear provenance. Workflow shapes change quickly; context alteration does not.

**What does the tool do?**  
Resolve declared packages, lock exact versions and hashes, render small deterministic outputs for different agent tools, and mark provisioned sections so humans and agents know what not to edit and where shared content comes from.

**Why now?**  
Broad cross-tool support for `AGENTS.md` and `.agents/` removed a major adoption blocker. Copy-paste still does not scale.

**Is the specification final?**  
No. This is an experiment. The repository and specification may be reworked as the model is validated. Contributions to the spec are especially welcome.

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
