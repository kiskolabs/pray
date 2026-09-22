# Canonical verification records

## What changed

- Added a canonical verification record model for package, render-plan, injected-bytes, and confession verification
- Clarified the stable fields needed to compare claims across heterogeneous clients, servers, and engines
- Kept verification records separate from package identity while still allowing metadata to support verification

## Why this matters

This gives Pray a stable, engine-agnostic way to prove what was claimed, what was verified, and what bytes were actually injected under zero-trust conditions.
