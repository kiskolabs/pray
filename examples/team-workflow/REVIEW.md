<!-- pray:0 ignore-comments -->

# Agent context

Do not edit managed blocks or managed skills.
Add or change project-specific instructions in `agent/local/` only.
To change shared guidance, ask a human to update `Prayfile` and run `pray`.

## Project-local instructions

### agent/local/project.md
Coordinate larger changes through a short checklist.
Call out any file that should not be rewritten automatically.

### agent/local/testing.md
Add a regression test for any change that touches rendered files, lockfiles, or recovery flows.

## Shared instructions

<!-- pray:5236da8c -->
When something breaks in production, stabilize first and investigate second.
Capture the reproduction path and keep the rollback steps explicit.
<!-- pray:5236da8c -->

<!-- pray:7191bb2f -->
Use focused regression tests for behavior that already failed once.
Keep user-facing file writes covered end-to-end.
<!-- pray:7191bb2f -->

<!-- pray:0cb2be24 -->
Review diffs for clarity first, behavior second, and copy last.
Prefer notes that help a teammate recover quickly if a change goes sideways.
<!-- pray:0cb2be24 -->

<!-- pray:32b8c448 -->
When a page is user-facing, verify the rendered result in addition to the source change.
Keep handoff instructions short and specific.
<!-- pray:32b8c448 -->
