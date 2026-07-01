# Distribution synchronization summary

This note captures the core distribution idea in one place:

- publish once
- install from a verified source
- sync between peers when availability needs more than one server
- verify every artifact with hashes and signatures
- keep provenance and resolved state in the lockfile

## What matters

The important contract is not the transport shape. The important contract is that the same package bytes, metadata, and verification records can move across machines without changing trust semantics.

That means:

- clients should see one install flow
- peers may mirror the same package set
- sync should be explicit and reviewable
- corruption or stale state should fail closed
- restored state should be reproducible from locked metadata

## How the current design fits

The current implementation uses a simple HTTP-based distribution point and optional peer synchronization. That is enough to prove the end-to-end contract for:

- publish
- consume
- sync
- verify

The beta E2E coverage now exercises that full path.
