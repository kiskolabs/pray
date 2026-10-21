# Isolate RubyGems release environment

## Participants

- Andrei Makarov

## Decisions

- Key the RubyGems release environment by the complete Ruby build and platform.
- Preserve the existing supported Ruby range. The repair isolates installed native extensions instead of requiring one local Ruby patch release.
- Add the environment regression check to the hosted Ruby job.

## Effects

- `scripts/release/rubygems.sh` now configures isolated RubyGems and Bundler paths before resolving the locked Bundler version or installing dependencies.
- User-installed gems cannot enter the release dependency path.
- A compatible later release run reuses the ignored per-build cache, while another Ruby patch release receives a separate cache.
- The full RubyGems dry-run passed on Ruby 3.4.7 in the environment that previously loaded a Ruby 3.4.10 native extension.

Validation completed with these commands and observed results:

- `bash scripts/release/rubygems_environment_test.sh`: passed after first failing because the isolation helper was absent, then failing on a relative package path before path canonicalization was added.
- `scripts/release/rubygems.sh --dry-run`: installed Bundler 4.0.15 and the locked dependencies in the isolated Ruby 3.4.7 cache, inspected 192 files without offenses, passed 331 examples, built `pkg/pray-cli-1.21.0.gem`, and pushed nothing.
- `bundle exec bundler-audit check --update` in the isolated environment: checked the updated advisory database and found no vulnerabilities.
- `make loc-check`: passed with existing allowlisted and warning-only files, and no unratcheted file above the hard limit.
- `cargo fmt --all --check`: passed.
- `bash -n scripts/release/common.sh scripts/release/rubygems.sh scripts/release/rubygems_environment_test.sh`: passed.
- `git diff --check`: passed.

## Next

- Confirm the new regression check in hosted CI.
- Publish the remaining RubyGems 1.21.0 surface.

## Source

- `docs/issues/20261021183640_isolate-rubygems-release-environment.md`
- `scripts/release/common.sh`
- `scripts/release/rubygems.sh`
- `scripts/release/rubygems_environment_test.sh`
