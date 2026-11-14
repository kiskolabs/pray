# Distribution synchronization summary

This note captures the core distribution idea in one place:

- publish once
- install from a verified source
- sync between peers when availability needs more than one server
- verify every artifact with hashes and signatures
- keep provenance and resolved state in the lockfile

## What matters

Package bytes, metadata, and verification records must retain their trust semantics when transferred between machines.

That means:

- clients should see one install flow
- peers may mirror the same package set
- sync should be explicit and reviewable
- corruption or stale state should fail closed
- restored state should be reproducible from locked metadata

## How the current design fits

The current implementation uses a simple HTTP-based distribution point and optional peer synchronization. The HTTP publish, consume, sync, and verify E2E coverage is recorded in [RFC 0060 implementation notes](../rfcs/0060-distribution.md#implementation-notes).

[RFC 0104](../rfcs/0104-federation-transports.md) defines peer relationships and the experimental federation wire.
