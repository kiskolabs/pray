# Install refreshes a stale git catalog for new packages

## Participants

Andrei Makarov

## Decisions

Install stays on a locked git revision when current packages still resolve. A newly declared package missing from that catalog is the same class of failure as a version the locked catalog cannot satisfy, so install refreshes the git source.

If refresh still cannot find the package, the error names the locked revision and pray update.

## Effects

Rust, Ruby, and TypeScript matchers treat not found in distribution and missing v1/packages metadata as refresh candidates. Git resolution rewrites that miss so it does not look like a typo in the package name.

## Next

Covered by install_refreshes_when_a_new_catalog_package_is_declared and the matching Ruby and TypeScript cases.

## Source

usr/docs/issues/20260908120000_install-git-catalog-refresh.md
CHANGELOG.md Unreleased
