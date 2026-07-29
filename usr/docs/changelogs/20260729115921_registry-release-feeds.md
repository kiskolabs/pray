# Registry release feeds

## Participants

- Andrei Makarov

## Decisions

- Document first-party release feeds next to the releasing checklist so operators can subscribe after publish.
- crates.io and RubyGems expose per-package feeds; npmjs does not.

## Effects

- Added a Registry release feeds section to docs/releasing.md with crates.io RSS and RubyGems Atom URLs for pray-cli, plus the npm gap.

## Next

- After RubyGems publish, confirm https://rubygems.org/gems/pray-cli/versions.atom returns Atom (404 until the gem exists).

## Source

- crates.io data access: https://crates.io/data-access
- RubyGems gem page alternate link: /gems/<name>/versions.atom
- npm registry global feed: https://registry.npmjs.org/-/rss
