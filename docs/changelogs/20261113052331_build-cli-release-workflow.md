# Build CLI release workflow

## Decisions

Ship an Actions workflow that builds unsigned platform CLI archives without requiring signing credentials. Keep signing optional. If the GitHub Release for the version tag exists, upload archives there; if missing, create the release from CHANGELOG notes (same contract as `make release-github`) then upload. Do not change `pray upgrade` or claim `pray.kisko.dev` as live binary origin yet.

## Effects

Added `.github/workflows/build-cli-release.yml` for macOS universal, Linux x86_64, and Windows x86_64 archives with VERSION, SOURCE_COMMIT, SIGNING, and checksum sidecars. Attach job creates or updates the matching GitHub Release. Documented mutual respect with `make release-github` in `docs/releasing.md` and `packaging/README.md`. Updated `docs/issues/20261027144200_signed-platform-binaries-on-tag.md`.

## Next

- Push annotated tag `v1.26.3` and run Build CLI release (or `make release-github` then the workflow) so notes and assets land together.
- Continue R2 index/RSS, local optional sign scripts, and `pray upgrade` preference work from that issue.
- Later pass (2026-11-13): dropped CI signing secrets; see `docs/changelogs/20261113075658_cli-release-local-signing-only.md`.

## Source

- docs/issues/20261027144200_signed-platform-binaries-on-tag.md
- .github/workflows/build-cli-release.yml
- docs/releasing.md
- packaging/README.md
