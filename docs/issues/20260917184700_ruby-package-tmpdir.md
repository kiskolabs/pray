# Ruby pray package missing tmpdir

## Participants

Andrei Makarov

## Decisions

Load tmpdir from the Ruby archive and publish libraries. Specs already required tmpdir, so in-process package tests passed. The installed gem CLI does not load spec_helper. After gem push, PATH pray is the RubyGems binstub, and make release-all distribution publish calls that binary.

Keep PRAY as the selector for distribution.sh. Print the resolved binary and version before package so a Ruby gem on PATH is visible. Do not change the 1.18.0 registry packages from this pass.

## Effects

A child ruby process that required pray without tmpdir raised undefined method mktmpdir for class Dir. After require tmpdir in archive.rb and publish.rb, that process exits 0. bundle exec rspec spec/pray/archive_spec.rb spec/pray/publish_spec.rb: 18 examples, 0 failures. bundle exec rubocop on those files: no offenses.

PRAY=$HOME/.cargo/bin/pray PRAY_RELEASE_YES=1 make release-distribution printed pray 1.18.0 and finished. It published packages/prayer-publisher to ./prayers.

## Next

Ship RubyGems 1.18.1. Leave crates.io and npm at 1.18.0. See usr/docs/issues/20260917185100_prepare-1-18-1-release.md.

## Source

scripts/release/distribution.sh
rubygems/pray-cli/lib/pray/archive.rb
rubygems/pray-cli/lib/pray/publish.rb
rubygems/pray-cli/spec/spec_helper.rb
usr/docs/changelogs/20260917184700_ruby-package-tmpdir.md
