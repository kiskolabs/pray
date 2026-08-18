# RFC 0102: Destination DSL as canonical examples

- Feature Name: destination-dsl-canonical
- Type: Standards Track
- Status: Experimental
- Created: 2026-08-17
- Author: Andrei Makarov
- Relates: RFC 0010, RFC 0030

## Summary

The recommended Prayfile surface is `compose`, `tree`, and `pray …, file:`. RFC 0010 still includes `target` / `output` / `agent` examples. This RFC proposes making destination DSL the canonical examples and treating legacy keywords as a migration appendix until CLI version 2.

## Motivation

New readers copy RFC 0010 examples and get deprecation warnings. The dual grammar also complicates polyglot fixtures (two shared cases: `compose-tree-file` and `legacy-target`).

## Guide-level explanation

A new Prayfile looks like `compose "AGENTS.md" do … end` and `tree ".agents/skills" do … end` with `pray "sample/base", "~> 1.4"`. `pray format` is the upgrade tool.

After this RFC is Stable, a Level 0 parser MUST accept destination DSL. Legacy keywords SHOULD warn and MUST keep parsing until the version 2 removal stated in changelog 1.6.0.

Keep a legacy fixture so parsers do not drop compatibility early.

## Reference-level explanation

Key words follow RFC 2119.

Rewrite RFC 0010 examples for the Prayfile surface to destination DSL. Move `target` / `output` / `agent` to a compatibility appendix with the version 2 removal note. Conflict policy is `fail` only (RFC 0030).

## Implementation notes

Warnings already fire. Shared corpus already has both shapes. This RFC is documentation honesty plus fixture labeling.

## Unresolved questions

Exact version 2 date. Whether `group` and `environment` examples belong in the same RFC 0010 pass (RFC 0103 reserved).
