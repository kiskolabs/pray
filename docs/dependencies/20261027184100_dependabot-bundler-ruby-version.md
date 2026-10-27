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

Add rubygems/pray-cli/.ruby-version with 3.4 so Dependabot matches CI. Keep the gemspec runtime floor at >= 3.1.0 unless product support for 3.1 is dropped. Comment the Dependabot bundler entry so the pin is not removed by accident.

## Next

Re-run or wait for the next scheduled bundler update and confirm a PR opens. Watch whether Dependabot images ship Ruby 3.4 for .ruby-version 3.4; if resolution still fails, pin an exact patch (for example 3.4.5) that the updater supports.

## Source

- .github/dependabot.yml
- .github/workflows/ci.yml (ruby-version 3.4)
- rubygems/pray-cli/Gemfile
- rubygems/pray-cli/pray-cli.gemspec
