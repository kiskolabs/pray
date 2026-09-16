# Pack and publish refuse duplicate archive paths

## Participants

Andrei Makarov

## Decisions

Refuse a second member with the same archive path at pack time. Do not skip or overwrite. If the repeated path is the auto-included package spec, say that spec.files must not list it.

Derive catalog metadata through the same unpack that consumers use, not a lenient tar unpack that overwrites a repeated path.

Do not reject *.prayspec in spec.files at parse time. Path packages that never pack would still work, and pack already fails.

No new RFC. RFC 0011 already requires rejecting duplicate normalized archive paths.

## Effects

pray package and pray publish fail with integrity exit 4 when spec.files repeats a content path. Catalog derive unpacks with unpack_praypkg, so a crafted archive with a repeated member cannot enter derived_metadata.

In-repo *.prayspec files do not list the spec. pray init already omitted it.

A later pass changed the listed-spec case. RFC 0114 requires a refreshed fork spec.files to include the fork prayspec path. Pack still auto-includes that file, so a second insert of that name is skipped once. A repeated content path still fails. Skip/overwrite of other members is not allowed.

A later pass keys pack and unpack on the normalized path. ./README.md and README.md are the same member. Current-directory segments are dropped before the uniqueness set.

Ruby and TypeScript pack now match that rule. They record member paths before writing the staging tree, skip the auto-included spec if spec.files lists it, and raise duplicate package archive path on any other repeat.

## Next

Optional later: reject *.prayspec in spec.files at parse time if authors keep listing it outside a fork refresh.

A repo checker that only inspects path packages would still miss a bad archive. Pack and catalog unpack are the gates.

Later pass 20260915103600: this work ships as 1.15.0. See usr/docs/issues/20260915103600_prepare-1-15-0-release.md.

## Source

RFC 0011 section 26 (duplicate normalized paths)
RFC 0114 (fork spec.files lists the prayspec)
crates/pray-cli/src/archive_members.rs
crates/pray-cli/src/materialize.rs
crates/pray-core/src/derived_metadata.rs
crates/pray-core/src/package_archive.rs
crates/pray-core/src/paths.rs
rubygems/pray-cli/lib/pray/archive.rb
npmjs/pray-cli/src/archive/praypkg.ts
usr/docs/changelogs/20260915084500_pack-duplicate-archive-paths.md
usr/docs/changelogs/20260915090000_align-upstream-publish-clients.md
usr/docs/issues/20260915103600_prepare-1-15-0-release.md
