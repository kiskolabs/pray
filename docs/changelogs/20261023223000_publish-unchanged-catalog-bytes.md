# Publish leaves unchanged catalog bytes alone

## Decisions

Unchanged `pray publish` versions do not rewrite package catalog JSON. A load and serialize round trip was stripping trailing newlines and reordering keys such as `artifact_hash` relative to `yanked`.

## Effects

Rust, Ruby, and TypeScript publish paths skip `write_registry_package_metadata` when the stored artifact still matches. Specs assert byte identity across republish, including catalogs that end with a newline.

## Next

Release after 1.22.0 so downstream publishers can drop the post-publish catalog restore step for this noise class.

## Source

Downstream report: amkisko/prayers docs/dependencies/20261023221634_pray-1-22-catalog-serializer-noise.md
