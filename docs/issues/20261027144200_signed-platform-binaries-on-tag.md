# Signed platform binaries on tag

## Participants

Andrei Makarov

## Decisions

- Goal: when an annotated version tag lands, publish signed `pray` CLI binaries for macOS, Linux, and Windows. Language-registry publish and CHANGELOG-backed release notes stay on the existing manual `scripts/release/` path.
- Binary publish origin (locked 2026-10-27, supersedes GitHub Release assets as the install origin): signed platform binaries publish under `https://pray.kisko.dev`, versioned by release. Keep the last 10 published versions at that origin; prune older version trees on publish. GitHub Releases remain for CHANGELOG-backed notes and version discovery unless a later decision moves discovery to the same origin.
- Binary host repository (locked 2026-10-27): `https://pray.kisko.dev` is GitHub Pages from [Prayfile/prayfile.github.io](https://github.com/Prayfile/prayfile.github.io) (`CNAME` = `pray.kisko.dev`). Signed binaries land in that repository (versioned paths under the Pages tree), not as GitHub Release assets on `kiskolabs/pray`. Publish scripts push or open a change against `prayfile.github.io`; prune keeps only the last 10 version directories in the published tree.
- This repository does not yet build, sign, or upload platform binaries on tag. `make release-github` / `scripts/release/github.sh` creates or edits release notes only. `pray upgrade` still installs via `cargo install`, not release assets.
- Required platform set: macOS (Apple Silicon and Intel), Linux (at least x86_64; aarch64 and musl optional until chosen), Windows (at least x86_64; aarch64 optional until chosen). Artifact naming and checksum files must be a fixed contract before the first tagged upload.
- Signing contract by platform (locked 2026-10-27): macOS uses Apple Developer ID Application codesign with notarization and staple; Windows uses Azure Trusted Signing; Linux uses Sigstore cosign keyless (OIDC) with no long-lived signing key in repo secrets.
- Preferred split (locked 2026-10-27): GitHub Actions may build unsigned platform artifacts (matrix for macOS, Linux, Windows). Signing, checksumming, and publishing to `pray.kisko.dev` stay local-first via `scripts/release/` (dry-run by default, explicit publish flag). Tag alone does not publish signed binaries. Operator downloads unsigned artifacts (workflow artifacts or draft release), signs, then uploads to the binary origin.
- Single signing host (locked 2026-10-27): it must be possible to sign all three platform binaries on one macOS instance. That host runs Apple codesign and notarization for macOS artifacts, Azure Trusted Signing for Windows artifacts, and cosign for Linux artifacts, then uploads. Scripts must not require a Windows or Linux machine for the normal operator path.
- Full end-to-end GitHub Actions (build plus sign plus upload) remains optional and not mandatory. If added later, it must reuse the same scripts and asset contract and must not be the only supported path. Local-only build without Actions must still work.
- Credentials for Apple Developer ID, notarization API, and Azure Artifact Signing (formerly Trusted Signing) stay with the operator on that macOS signing path. Cosign keyless from that Mac uses interactive OIDC (browser), not GitHub Actions OIDC; consumers verify against the operator identity and issuer (see Findings).
- `pray upgrade` must prefer a signed binary from `pray.kisko.dev` for the running platform: download, verify signature (and checksum), then install. Fall back to `cargo install` and to building from source only when no matching verified asset is available.
- Homebrew and AUR stay on source tarball until an explicit later decision retargets them at prebuilt assets.

## Findings

Research 2026-10-27 on signing all three platforms from one macOS host.

Windows / Azure Artifact Signing (formerly Trusted Signing):

- Microsoft official SignTool plus Azure.CodeSigning.Dlib path requires Windows (Windows 10 1809+, SDK SignTool, .NET 8, VC++ redistributable). Learn docs list SignTool, GitHub Actions, Azure DevOps, PowerShell, and the Artifact Signing SDK; they do not list a first-party macOS PE client.
- Cross-platform Authenticode against the same Azure service is documented by third-party tools that run on macOS without Wine. Primary candidate: Jsign (`--storetype TRUSTEDSIGNING`, Homebrew on macOS/Linux), with Azure CLI token (`az account get-access-token --resource https://codesigning.azure.net`) or equivalent. Also cited: SignUniversal.Cli (.NET tool), OSSign, Velopack 1.3+, electron-builder azure backend (may use Wine for signtool).
- Verdict: feasible on macOS via a third-party client (prefer Jsign unless dependency-policy rejects JVM). Not feasible via Microsoft SignTool alone on Darwin. Smoke-test with a real Artifact Signing account remains required before locking the script dependency.

Linux / cosign keyless:

- Cosign ships macOS binaries and Homebrew installs. `cosign sign-blob` without `--key` is keyless: Fulcio issues a short-lived cert after interactive OIDC (Google, GitHub, or Microsoft browser flow). Bundle file holds signature, cert, and Rekor proof.
- This works on a developer Mac; it does not require GitHub Actions. The certificate identity is the operator email (or provider subject), not a workflow URI. `pray upgrade` and docs must pin `--certificate-identity` and `--certificate-oidc-issuer` (or regex) to that identity. CI keyless (token.actions.githubusercontent.com) is a different identity and must not be mixed without an explicit dual-trust policy.
- Verdict: keyless from macOS is supported by Sigstore docs. Remaining work is identity policy and a real sign/verify smoke-test of the Linux release blob from the operator Mac.

macOS:

- Unchanged: codesign, notarytool, staple already require macOS; no research conflict.

## Effects

- None in tree yet. This note records the gap and the acceptance contract only. Signing path, binary origin on `pray.kisko.dev` with last-10 retention, upgrade preference, local-first signing on one macOS host, and optional CI build of unsigned artifacts were locked in Decisions above; implementation has not started. Findings above close the earlier open question that cosign keyless could only run in Actions.

## Next

- Freeze the HTTP path contract under `pray.kisko.dev` / `prayfile.github.io` (version directory layout, artifact names, checksum and signature sidecars, latest pointer or index, and prune of versions beyond the last 10). Prefer a path that stays off the landing page root clutter (candidate: `releases/v{VERSION}/`).
- Decide how binaries enter `prayfile.github.io` (direct commit of last-10 trees versus Git LFS versus publish from release CI into that repo) so git history size stays operable under GitHub soft limits while Pages serves the tip.
- Add local-first release scripts (and Makefile targets) that run on macOS to fetch or build unsigned artifacts, sign all three platforms, checksum, publish into `Prayfile/prayfile.github.io`, and prune older than the last 10 versions; document credentials and dry-run versus publish in `docs/releasing.md`, `scripts/release/README.md`, and `packaging/README.md`.
- Optionally add a GitHub Actions workflow that only builds and publishes unsigned artifacts for the operator to sign on macOS; do not require sign-in-CI.
- Choose the Windows-on-macOS client (default lean: Jsign) under dependency-policy; smoke-test Artifact Signing of a PE from Darwin; smoke-test cosign sign-blob keyless of a Linux artifact from the same Mac; freeze the verify identity and issuer strings for `pray upgrade`.
- Provision Apple Developer ID plus notarization API credentials and an Azure Artifact Signing account plus Certificate Profile Signer role; document the cosign identity consumers must verify.
- Implement `pray upgrade` to select the matching asset from `pray.kisko.dev`, verify, install, and only then fall back to `cargo install` or source.
- Prove one dry-run then a real publish of signed assets to `pray.kisko.dev` via the macOS scripts before treating the path as production-ready.

## Source

- docs/releasing.md
- scripts/release/README.md
- scripts/release/github.sh
- packaging/README.md
- crates/pray-cli/src/cli_release.rs
- docs/issues/20261026053110_jsr-and-linux-os-packaging.md
- https://github.com/Prayfile/prayfile.github.io
- https://pray.kisko.dev/
- https://learn.microsoft.com/en-us/azure/artifact-signing/how-to-signing-integrations
- https://ebourg.github.io/jsign/
- https://stackoverflow.com/questions/78790089/how-can-i-invoke-azure-trusted-signing-from-a-linux-os
- https://docs.sigstore.dev/quickstart/quickstart-cosign/
- https://docs.velopack.io/packaging/signing
