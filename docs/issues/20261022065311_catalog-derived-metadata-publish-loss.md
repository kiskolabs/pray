## Participants

Andrei Makarov

## Decisions

Ruby and TypeScript `pray publish` must round-trip `derived_metadata` on every catalog version row and derive it for newly published rows, matching the Rust CLI and `schema/registry.schema.json`. Tip-only storage remains a separate size plan, not a reason to drop existing fields.

## Effects

- Confirmed the active shell `pray` for the downstream report was the Ruby gem 1.21.0, not the Rust binary.
- Added load and write support plus root-based derivation in Ruby and TypeScript.
- Publish specs cover unchanged republish preservation and historical row retention when appending a version.

## Next

- Ship the fix in the next Ruby and npm release.
- Downstream catalogs already repaired from Git can stay; new publishes should use the fixed CLI or Rust.

## Source

Dependency note: `docs/dependencies/20261022065311_pray-ruby-npm-catalog-derived-metadata-loss.md`
Multi-model audit: catalog merge, contract lineage, publish regression hunt.
