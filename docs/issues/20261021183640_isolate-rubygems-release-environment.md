# Isolate RubyGems release environment

## Participants

- Andrei Makarov

## Decisions

- Isolate release gems and Bundler state by Ruby engine, complete Ruby version, patch level, architecture, and platform.
- Ignore the operator's user gem path during the RubyGems release.
- Keep the per-build environment under the ignored `rubygems/pray-cli/tmp/` tree so a later run can reuse compatible native extensions.
- Do not publish from the repair run. Validate the complete RubyGems path with `--dry-run` first.

## Effects

- Ruby 3.4.7 no longer loads the `date` native extension installed by Ruby 3.4.10 from the shared user gem directory.
- The regression check starts with deliberately polluted gem and Bundler paths, configures the release environment, and verifies that RubyGems sees only the per-build gem home.
- The check also invokes the helper with a relative package path from another working directory. This exposed and then verified the package-path canonicalization used by the release cache.
- The RubyGems dry-run installed Bundler 4.0.15 and the locked dependencies in the Ruby 3.4.7 cache, inspected 192 Ruby files without offenses, passed 331 examples, and built `pray-cli-1.21.0.gem` without pushing it.
- The updated Ruby advisory database reported no vulnerabilities in the locked gem graph.
- Hosted CI passed the Ruby, npm, and Rust jobs, including the new release-environment check, coverage floors, mutation smoke, and SonarQube scan. The separate Scorecard analysis also passed.
- Registry checks on 2026-10-21 found `pray-core`, `pray-transport`, and the Rust `pray-cli` at 1.21.0 on crates.io, npm `pray-cli` at 1.21.0, and RubyGems `pray-cli` at 1.20.0. The RubyGems surface remains unpublished.
- The operator-facing surface is the terminal release flow. The successful dry-run reaches the artifact summary instead of a Bundler error report. There are no interactive states before the existing publish confirmation. A narrow terminal wraps dependency-install output but preserves the failure or completion line. A person can identify the artifact and whether it was pushed.

## Next

- Publish only the RubyGems surface after the repair is on `main`.
- Create the GitHub release after all registry surfaces report 1.21.0.

## Source

- `scripts/release/rubygems.sh`
- `scripts/release/common.sh`
- `scripts/release/rubygems_environment_test.sh`
- `scripts/release/README.md`
- https://crates.io/crates/pray-core/1.21.0
- https://crates.io/crates/pray-transport/1.21.0
- https://crates.io/crates/pray-cli/1.21.0
- https://www.npmjs.com/package/pray-cli/v/1.21.0
- https://rubygems.org/gems/pray-cli/versions/1.20.0
