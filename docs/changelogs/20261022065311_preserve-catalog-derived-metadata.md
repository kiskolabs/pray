## Participants

Andrei Makarov

## Decisions

Ruby and TypeScript publishers carry `derived_metadata` through catalog load and write, and derive it for new version rows from the package root.

## Effects

Publish no longer drops historical derived metadata when rewriting package catalogs. New rows include summary, topics, categories, effects, embeddings, and counts.

## Next

Release Ruby and npm packages that include the fix.

## Source

`docs/issues/20261022065311_catalog-derived-metadata-publish-loss.md`
`docs/dependencies/20261022065311_pray-ruby-npm-catalog-derived-metadata-loss.md`
