# Dependabot bundler resolver used Ruby 3.1

## Dependency

bundler ecosystem under rubygems/pray-cli (Dependabot Updates workflow).
Development gems: polyrun ~> 2.2, bundler-audit ~> 0.9, rbs ~> 3.9.
pray-cli.gemspec required_ruby_version remains >= 3.1.0 for the published gem.

## Symptom

Weekly Dependabot bundler jobs fail with dependency_file_not_resolvable. The updater runs Ruby 3.1.7 while polyrun (>= 2.2.2) needs Ruby >= 3.4, and bundler-audit / related gems need Ruby >= 3.2. No pull request is opened for the bundler group.

## Evidence

- https://github.com/kiskolabs/pray/actions/runs/40349507772
- https://github.com/kiskolabs/pray/actions/runs/40259787874
- Updater log: current Ruby version is = 3.1.7; polyrun = 2.2.5 and rbs = 4.2.0 candidates fail the same way.
- CI ruby job already pins ruby-version: "3.4" in .github/workflows/ci.yml.
- No .ruby-version or .tool-versions existed under rubygems/pray-cli, so Dependabot fell back to the gemspec floor.

## Suggested fix

Pin rubygems/pray-cli/.ruby-version to an exact installed patch that matches CI (currently 4.0.7). Keep the gemspec runtime floor at >= 3.1.0 unless product support for 3.1 is dropped. Comment the Dependabot bundler entry so the pin is not removed by accident.

A bare major.minor value such as 3.4 satisfies GitHub Actions ruby/setup-ruby, but local rbenv treats it as a missing version and aborts make release-rubygems / release-all when the package directory is entered.

## Effects

- rubygems/pray-cli/.ruby-version, CI ruby-version, and the Dependabot comment now use 4.0.7.
- make release-all for 1.26.0 failed at the RubyGems surface with rbenv: version `3.4' is not installed while .ruby-version still held 3.4. Reproduced locally with a temporary .ruby-version containing 3.4.
- RubyGems later lists pray-cli 1.26.0; resume skips an already-published gem version.

## Next

Confirm the next Dependabot bundler PR opens against the 4.0.7 pin. Keep .ruby-version as an exact patch whenever CI or local rbenv pins change.

## Source

- .github/dependabot.yml
- .github/workflows/ci.yml (ruby-version 4.0.7)
- rubygems/pray-cli/.ruby-version
- rubygems/pray-cli/Gemfile
- rubygems/pray-cli/pray-cli.gemspec
- scripts/release/rubygems.sh
