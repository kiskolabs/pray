## Decisions

Normalize archive member paths before the uniqueness set. Drop current-directory segments. Pack writes the normalized name. Unpack and catalog derive use the same rule.

Port pray update --latest to the Ruby CLI, including --latest --dry-run. --json and --major stay rejected.

## Effects

spec.files listing ./README.md and README.md fails pack with duplicate package archive path.

Ruby pray update --latest rewrites an exact spec.upstream pin and refreshes the path tree. --latest --dry-run prints the planned pin and does not write.

## Next

Optional: reject *.prayspec in spec.files at parse time.

Later pass 20260915103600: this work ships as 1.15.0. See usr/docs/issues/20260915103600_prepare-1-15-0-release.md.

## Source

usr/docs/issues/20260915084500_pack-duplicate-archive-paths.md
RFC 0011 section 26
RFC 0114
CHANGELOG.md 1.15.0
usr/docs/issues/20260915103600_prepare-1-15-0-release.md
