## Project-local instructions

### agent/local/project.md
Keep compliance-facing changes auditable.
Call out any place where manual approval is required.

### agent/local/security.md
Treat any external input as untrusted until it is verified.
Never hide a manual review step behind automation.

## Shared instructions

<!-- pray:2cb9d907 -->
Keep audit notes terse, factual, and traceable.
Record the reason a change is allowed to land.
<!-- pray:2cb9d907 -->

<!-- pray:03b7305e -->
Every risky change should include a rollback path.
Prefer a plan that can be executed without rewriting the whole tree.
<!-- pray:03b7305e -->
