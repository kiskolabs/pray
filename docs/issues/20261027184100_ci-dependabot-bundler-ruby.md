# CI Dependabot bundler Ruby pin

## Participants

Andrei Makarov

## Decisions

Pin Dependabot bundler resolution for rubygems/pray-cli to Ruby 3.4 via .ruby-version, matching the CI ruby job. Do not raise the published gemspec required_ruby_version floor in this change.

## Effects

Main CI workflow on HEAD (bed67ce) already passes rust, ruby, npm, and Scorecard. The most recent main CI failure (run 40357457910) was loc-check on npmjs/pray-cli/src/cli/main.ts at 303 lines; later commits cleared that hard limit. Remaining red Actions runs were Dependabot bundler updates failing on Ruby 3.1.7. Added rubygems/pray-cli/.ruby-version and a Dependabot comment; recorded the upstream-facing note under docs/dependencies/.

## Next

Push the pin and confirm the next bundler Dependabot run resolves. No product CI code change required for the closed loc-check failure.

## Source

- https://github.com/kiskolabs/pray/actions/runs/40508001910 (latest main CI success)
- https://github.com/kiskolabs/pray/actions/runs/40357457910 (loc-check failure, superseded)
- https://github.com/kiskolabs/pray/actions/runs/40349507772 (Dependabot bundler)
- docs/dependencies/20261027184100_dependabot-bundler-ruby-version.md
