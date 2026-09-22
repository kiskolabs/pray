# Ruby pray package missing tmpdir

## Participants

Andrei Makarov

## Decisions

Require tmpdir in the Ruby archive and publish libraries. Specs loaded tmpdir through spec_helper, so in-process tests missed the CLI path.

## Effects

pray package on the installed RubyGems CLI raised undefined method mktmpdir for class Dir during make release-all distribution publish. A subprocess that requires pray without tmpdir now defines Dir.mktmpdir.

distribution.sh prints the resolved pray path and version before package.

PRAY=$HOME/.cargo/bin/pray PRAY_RELEASE_YES=1 make release-distribution finished for ./prayers.

## Next

Ship the Ruby load on 1.18.1. Leave crates.io and npm at 1.18.0.

## Source

usr/docs/issues/20260917184700_ruby-package-tmpdir.md
rubygems/pray-cli/lib/pray/archive.rb
rubygems/pray-cli/lib/pray/publish.rb
scripts/release/distribution.sh
