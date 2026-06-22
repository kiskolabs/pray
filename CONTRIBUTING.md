# Contributing Guidelines

Thank you for your interest in contributing to the Agentfile open specification and the planned **pata** reference implementation. This repository is **spec-first**: polish the open specification before implementing the changes. We value learning over perfection but require rigor and responsibility.

## The Golden Rule of Automation

We welcome the use of AI and automation tools to reduce toil, but you must strictly adhere to the following:

1. You are the Author: You act as the responsible agent for any code you submit. You must review, debug, and understand every line.

2. Manage Cognitive Load: Do not submit massive, unreviewed automated dumps. Respect the reviewers' time by annotating complex logic.

3. Security: Never feed project secrets or private context into public AI models.

## What to change where

| Change | Where |
|--------|-------|
| Normative specification | [SPEC.md](SPEC.md) |
| Repository workflow for humans and agents | [AGENTS.md](AGENTS.md) |
| Test coverage expectations | [spec/README.md](spec/README.md) |
| Reference CLI and libraries | reference implementation workspace (when present) |

Specification changes should stay language-independent and implementation-independent unless you are documenting a deliberate conformance extension.

## Branch naming

| Kind | Pattern |
|------|---------|
| Feature | `feature/<title>` |
| Bugfix or chore | `patch/<title>` |
| Release candidate or integration before `main` | `trunk/<title>` |
| Exploration or ideation | `plan/<title>` |

## How to Contribute

### 1. Reporting Issues

* Verify Accuracy: Before posting, verify your information. Avoid generalizations.

* Use Structured Inputs: Use our Issue Templates to provide clear goals, constraints, and reproduction steps. This helps us understand the context immediately.

### 2. Pull Request Process

* Scope: Keep PRs focused on a single goal.

* Context: Explain *why* the change is necessary. Transparency builds trust.

* Testing: Run focused tests, then the full suite. List exact commands run and observed results in validation output. Follow [spec/README.md](spec/README.md).

Pull request descriptions should answer:

- What problem is solved
- Why it matters
- How the solution works
- Any relevant context

For non-trivial changes, include reproduction steps or a changelog entry with intent.

### 3. Review Process

* We encourage productive friction. Expect questions about your approach.

* If a reviewer suggests a change, view it as mutual aid, not criticism.

## Checks

When the reference implementation workspace is present:

```bash
cargo build --workspace
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
```

For specification-only changes, review [SPEC.md](SPEC.md) for internal consistency and update examples or fixtures when behavior changes.

## Security

Report security issues privately; see [SECURITY.md](SECURITY.md).
