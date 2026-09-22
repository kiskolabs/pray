# Storefront search hook

## Participants

- Andrei Makarov

## Decisions

- Put the positioning line package manager for the language placed before inference into registry description and summary fields, not only README openers.
- crates.io search weights name, then keywords, then description, then readme; README-only placement was not enough for that query.
- Bump to 1.5.2 so crates.io can refresh published metadata after 1.5.1.

## Effects

- Updated pray-cli and pray-core Cargo descriptions; npm package description; RubyGems summary and description.
- Synced workspace version surfaces to 1.5.2.

## Next

- Publish 1.5.2 to crates.io, npmjs, and RubyGems.
- Tag v1.5.2 and bump Homebrew after publish.
- Confirm crates.io search for language before inference returns pray-cli.

## Source

- Branch: patch/storefront-search-hook
- Prior: usr/docs/changelogs/20260729103151_prepare-registry-publishing.md
