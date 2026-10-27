# Signed platform binaries on tag

## Participants

Andrei Makarov

## Decisions

- Goal: when an annotated version tag lands, publish signed `pray` CLI binaries for macOS, Linux, and Windows. Language-registry publish and CHANGELOG-backed release notes stay on the existing manual `scripts/release/` path.
- Binary publish origin (locked 2026-10-27, supersedes GitHub Release assets as the install origin): platform binaries, release index, and release RSS publish under `https://pray.kisko.dev`, versioned by release. Keep the last 10 published versions at that origin; prune older version trees on publish. GitHub Releases remain for CHANGELOG-backed notes on `kiskolabs/pray` unless a later decision drops them; they are not the binary install origin.
- Site and download point (locked 2026-10-27): `https://pray.kisko.dev` is the Cloudflare-fronted site whose landing today comes from [Prayfile/prayfile.github.io](https://github.com/Prayfile/prayfile.github.io) (`CNAME` = `pray.kisko.dev`). The download point is the same domain: a fresh index, an RSS feed, and versioned binary objects. Prefer object storage (Cloudflare R2 behind the same hostname or a `releases` subdomain) for binary bytes so git history does not hold the last-10 trees; index and RSS may live on Pages or beside the objects. Do not treat committed binaries in `prayfile.github.io` as the production path.
- Operator publish flow (locked 2026-10-27):
  1. GitHub Actions builds unsigned platform binaries (matrix) and stores them as workflow artifacts (or equivalent fetchable unsigned blobs). Tag alone does not publish to `pray.kisko.dev`.
  2. Operator runs a local `scripts/release/` path (dry-run by default, explicit publish flag): download those unsigned binaries, optionally codesign / notarize / Trusted Signing / cosign per platform, checksum, upload binary objects, update the public index and RSS, prune older than the last 10 versions.
  3. Users and `pray upgrade` consume the fresh index, RSS, and binaries from `pray.kisko.dev`.
- This repository does not yet build, sign, or upload platform binaries on tag. `make release-github` / `scripts/release/github.sh` creates or edits release notes only. `pray upgrade` still installs via `cargo install`, not release assets.
- Required platform set: macOS (Apple Silicon and Intel), Linux (at least x86_64; aarch64 and musl optional until chosen), Windows (at least x86_64; aarch64 optional until chosen). Artifact naming and checksum files must be a fixed contract before the first tagged upload.
- Signing contract by platform (locked 2026-10-27): when the operator enables signing, macOS uses Apple Developer ID Application codesign with notarization and staple; Windows uses Azure Trusted Signing; Linux uses Sigstore cosign keyless (OIDC) with no long-lived signing key in repo secrets. Signing and notarization are optional steps in the operator script (unsigned publish remains possible for dry-run and early smoke tests; production consumer verify policy still to freeze).
- Single signing host (locked 2026-10-27): when signing is used, it must be possible to sign all three platform binaries on one macOS instance. That host runs Apple codesign and notarization for macOS artifacts, Azure Trusted Signing for Windows artifacts, and cosign for Linux artifacts, then uploads. Scripts must not require a Windows or Linux machine for the normal operator path.
- Full end-to-end GitHub Actions (build plus sign plus upload) remains optional and not mandatory. If added later, it must reuse the same scripts and asset contract and must not be the only supported path. Local-only build without Actions must still work as a fallback input to the same operator script.
- Credentials for Apple Developer ID, notarization API, and Azure Artifact Signing (formerly Trusted Signing) stay with the operator on that macOS signing path. Cosign keyless from that Mac uses interactive OIDC (browser), not GitHub Actions OIDC; consumers verify against the operator identity and issuer (see Findings).
- `pray upgrade` must prefer a binary from the `pray.kisko.dev` index for the running platform: download, verify signature (and checksum) when the published policy requires it, then install. Fall back to `cargo install` and to building from source only when no matching verified asset is available.
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

- None in tree yet. This note records the gap and the acceptance contract only. Operator flow (Actions unsigned build → local script download / optional sign / upload / index+RSS update → users consume), binary origin on `pray.kisko.dev` with last-10 retention, R2-preferred object storage, upgrade preference, and single macOS signing host were locked in Decisions above; implementation has not started. Findings above close the earlier open question that cosign keyless could only run in Actions.

## Next

- Freeze the HTTP contract under `pray.kisko.dev`: versioned binary paths, checksum and signature sidecars, index document shape, RSS feed shape and URL, and prune of versions beyond the last 10 (candidate: `releases/v{VERSION}/`, `releases/index.json`, `releases/feed.xml`).
- Provision Cloudflare R2 (and Worker or `releases` subdomain if path split is chosen) for binary objects; keep landing on the existing Pages site; decide whether index and RSS are written into Pages, into R2, or both.
- Add GitHub Actions workflow that only builds and uploads unsigned platform artifacts for the operator to fetch; do not sign in CI.
- Add local-first release scripts (and Makefile targets) that download those artifacts, optionally sign/notarize, checksum, upload objects, rewrite index and RSS, and prune older than the last 10 versions; dry-run by default; document credentials and publish flag in `docs/releasing.md`, `scripts/release/README.md`, and `packaging/README.md`.
- Choose the Windows-on-macOS client (default lean: Jsign) under dependency-policy; smoke-test Artifact Signing of a PE from Darwin; smoke-test cosign sign-blob keyless of a Linux artifact from the same Mac; freeze production verify policy (signed-required versus allow unsigned) and identity strings for `pray upgrade`.
- Provision Apple Developer ID plus notarization API credentials and an Azure Artifact Signing account plus Certificate Profile Signer role; document the cosign identity consumers must verify.
- Implement `pray upgrade` to read the `pray.kisko.dev` index, select the matching asset, verify per policy, install, and only then fall back to `cargo install` or source.
- Prove one dry-run then a real publish (index + RSS + binaries) to `pray.kisko.dev` via the macOS scripts before treating the path as production-ready.

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
