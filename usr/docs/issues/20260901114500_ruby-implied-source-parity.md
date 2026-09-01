# Ruby implied source parity

## Participants

Andrei Makarov

## Decisions

Ruby ResolveSource.implied_source_name now matches Rust and TypeScript: explicit source: wins; otherwise the package namespace matching a source handle; otherwise the sole source; otherwise require source: when multiple sources exist.

## Effects

Prayfiles that omit source: when the namespace matches a declared source resolve on Ruby pray-cli. format_manifest omission of source: no longer breaks Ruby install for those packages.

## Next

None for this gap.

## Source

rfcs/0010-core-formats.md
crates/pray-core/src/resolve.rs implied_source_name
npmjs/pray-cli/src/resolve/package-root.ts impliedSourceName
rubygems/pray-cli/lib/pray/resolve_source.rb
rubygems/pray-cli/lib/pray/resolve.rb
usr/docs/changelogs/20260901113000_ruby-praypkg-unpack-and-cache-ready.md
