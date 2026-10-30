# CI Dependabot bundler Ruby pin

## Participants

Andrei Makarov

## Decisions

Pin Dependabot bundler resolution for rubygems/pray-cli via an exact .ruby-version patch that matches the CI ruby job. Do not raise the published gemspec required_ruby_version floor for this Dependabot fix. Later superseded the initial 3.4 pin with 4.0.7 so local rbenv and CI share one installed version.

## Effects

Main CI workflow on HEAD (bed67ce) already passes rust, ruby, npm, and Scorecard. The most recent main CI failure (run 40357457910) was loc-check on npmjs/pray-cli/src/cli/main.ts at 303 lines; later commits cleared that hard limit. Remaining red Actions runs were Dependabot bundler updates failing on Ruby 3.1.7. Added rubygems/pray-cli/.ruby-version and a Dependabot comment; recorded the upstream-facing note under docs/dependencies/.

A later release-all run for 1.26.0 failed on the RubyGems surface because rbenv rejects a bare 3.4 .ruby-version. The pin and CI job now use 4.0.7.

## Next

Confirm the next bundler Dependabot run resolves against 4.0.7. Keep .ruby-version exact when the CI Ruby pin moves.

## Source

- https://github.com/kiskolabs/pray/actions/runs/40508001910 (latest main CI success)
- https://github.com/kiskolabs/pray/actions/runs/40357457910 (loc-check failure, superseded)
- https://github.com/kiskolabs/pray/actions/runs/40349507772 (Dependabot bundler)
- docs/dependencies/20261027184100_dependabot-bundler-ruby-version.md
