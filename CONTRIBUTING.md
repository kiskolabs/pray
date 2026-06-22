# Contributing to Agentfile

Thank you for contributing to the Agentfile open specification and the **pata** reference implementation.

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

## Checks

When the reference implementation workspace is present:

```bash
cargo build --workspace
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
```

For specification-only changes, review [SPEC.md](SPEC.md) for internal consistency and update examples or fixtures when behavior changes.

## Tests

Follow [spec/README.md](spec/README.md):

1. Add or update a test that fails on the current bug or missing contract.
2. Implement the fix.
3. Run focused tests, then the full suite.
4. List exact commands run and observed results in validation output.

## Pull requests

Pull request descriptions should answer:

- What problem is solved
- Why it matters
- How the solution works
- Any relevant context

For non-trivial changes, include reproduction steps or a changelog entry with intent.

## Security

Report security issues privately; see [SECURITY.md](SECURITY.md).
