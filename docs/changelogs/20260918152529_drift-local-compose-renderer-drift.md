# Keep dest equal to a fresh render after adding a trailing local compose fragment

## Participants

vesan

Andrei Makarov

## Decisions

patch_rendered_content now copies unmarked text that sits after the last overlapping span when it appends a new managed span, and restores fresh unmarked text between adjacent dest spans when dest has none. Rust, Ruby, and TypeScript share that contract.

## Effects

pray drift no longer reports renderer_drift immediately after pray install when a compose block adds a local file after a package that dest already had.

## Next

Ship in the next CLI release. Cite GitHub issue 28.

## Source

usr/docs/issues/20260918152529_drift-local-compose-renderer-drift.md

https://github.com/kiskolabs/pray/issues/28
