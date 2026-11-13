# CLI release local signing only

## Participants

Andrei Makarov

## Decisions

Keep Build CLI release as unsigned CI only. Codesign, notarization, Trusted Signing, and cosign run on the operator host against the three CI archives. Actions must not hold Apple certificate or notary secrets. Keep the `SIGNING` sidecar as the local-sign anchor (`status=unsigned`, `sign_path=local`) and anchor public identities from repository variables (`team_id`, `cosign_identity`, `signer_subject`).

## Effects

Removed `attempt_sign` and macOS certificate / notary secret injection from `.github/workflows/build-cli-release.yml`. Each platform job writes an unsigned `SIGNING` sidecar with the matching identity field from repo vars (warns when empty). Superseded the single-macOS-only and optional CI-signing next lines in `docs/issues/20261027144200_signed-platform-binaries-on-tag.md`. Aligned `docs/releasing.md`, `packaging/README.md`, and `scripts/release/README.md`.

## Next

- Set repository variables `MACOS_TEAM_ID`, `LINUX_COSIGN_IDENTITY`, and `WINDOWS_SIGNER_SUBJECT` so CI archives carry non-blank identity anchors.
- Add local-first sign/publish scripts that download CI archives and optionally sign on macOS, Windows, or Linux as tools allow.
- Continue R2 index/RSS and `pray upgrade` preference work from that issue.

## Source

- docs/issues/20261027144200_signed-platform-binaries-on-tag.md
- .github/workflows/build-cli-release.yml
- docs/releasing.md
- packaging/README.md
- scripts/release/README.md
