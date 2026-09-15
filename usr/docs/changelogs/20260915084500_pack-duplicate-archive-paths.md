## Decisions

Pack records the normalized archive member path and fails on a second insert. Current-directory segments such as ./README.md collapse to README.md. The auto-included fork prayspec is skipped once when spec.files lists it after an RFC 0114 refresh. Other repeats use duplicate package archive path.

Catalog derive unpacks with unpack_praypkg.

## Effects

pray package and pray publish fail with integrity exit 4 when spec.files repeats a path after normalization. A listed auto-included spec still packs once.

A unique archive still derives catalog metadata. A crafted duplicate or aliased archive fails derive and unpack.

## Next

Optional later: reject *.prayspec in spec.files at parse time.

## Source

usr/docs/issues/20260915084500_pack-duplicate-archive-paths.md
RFC 0011 section 26
CHANGELOG.md Unreleased
