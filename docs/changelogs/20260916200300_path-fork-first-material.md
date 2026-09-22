# Path-fork first material and overlay files

## Participants

Andrei Makarov

## Decisions

Empty path-fork trees copy upstream content on install and update. Overlay extras live in the same path tree and spec.files. Outdated lists fork file drift against the locked upstream.

A later pass stopped listing the prayspec in spec.files. Identity is the file on disk. Empty spec.files is the starter. Refresh writes content paths only.

## Effects

pray install copies upstream files when the path package has no content yet. pray update still merges and keeps listed overlay files. pray outdated prints those paths under Outdated path forks.

## Next

Ships as 1.17.0. See usr/docs/issues/20260916223500_prepare-1-17-0-release.md. Plan would-copy listing stays open.

## Source

rfcs/0116-path-fork-first-material.md
usr/docs/issues/20260916200300_path-fork-first-material.md
