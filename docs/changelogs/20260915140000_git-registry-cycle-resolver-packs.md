# RFC 0100 git, registry, and cycle resolver packs

## Participants

Andrei Makarov

## Decisions

Ship git-distribution and registry-distribution lock slices plus a dependency-cycle reject pack. Port cycle reject to Ruby and TypeScript. Join relative git and registry source paths with the project root.

## Effects

All three libraries refuse sample/alpha and sample/beta cycling. They resolve sample/base from a committed local distribution for both git+file://distribution and a filesystem registry source.

## Next

PyO3 after RFC 0109 field freeze. Mix when a Phoenix repo asks. Tarball resolver pack.

Later pass 20260915144000: this work ships as 1.16.0. See usr/docs/issues/20260915144000_prepare-1-16-0-release.md.

## Source

usr/docs/issues/20260915140000_git-registry-cycle-resolver-packs.md
rfcs/0100-conformance.md
fixtures/resolver/git-distribution/expected.json
fixtures/resolver/registry-distribution/expected.json
fixtures/resolver/dependency-cycle/expected.json
CHANGELOG.md 1.16.0
usr/docs/issues/20260915144000_prepare-1-16-0-release.md
