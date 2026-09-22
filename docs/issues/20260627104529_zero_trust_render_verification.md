# Zero-Trust Render Verification and Engine-Agnostic Annotations

## Overview

Pray should treat all metadata, summaries, scores, confessions, and derived annotations as claims rather than truth.

The actual security boundary is the exact injected output. Package bytes, render plans, and final rendered bytes must be verified separately.

## Motivation

Different clients and servers may use manual review, hardcoded logic, heuristics, local inference, cloud inference, or generative models. The specification should remain valid across all of them.

## Scope

- Define zero-trust assumptions for clients, servers, and federation peers
- Clarify that annotations are claims with provenance, not truth
- Allow metadata to support verification of injected bytes
- Require verification of final rendered output