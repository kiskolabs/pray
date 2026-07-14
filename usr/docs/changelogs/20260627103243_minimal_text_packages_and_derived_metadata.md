# Minimal text packages and derived metadata

## What changed

- Clarified that a Pray package may consist only of minimal editable text files plus the required `*.prayspec`
- Added derived package metadata fields for language, encoding, origin, summary, categories, topics, counts, effects, side effects, and embeddings
- Added confession collection and relay semantics for publishers and federated servers

## Why this matters

This makes the package payload smaller and more durable, while letting distribution points compute richer annotations without changing package identity.

It also gives publishers a place to collect usage feedback across direct publishing and server-to-server synchronization.
