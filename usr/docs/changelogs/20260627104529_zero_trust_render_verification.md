# Zero-trust render verification

## What changed

- Added a zero-trust verification model for Pray packages, metadata, confessions, and federation peers
- Clarified that annotations may come from manual review, heuristics, local inference, cloud inference, or generative models
- Added render-digest verification as the security boundary for injected bytes

## Why this matters

This makes the spec resilient to untrusted servers and heterogeneous inference engines while keeping package identity separate from the exact bytes injected into target tools.
