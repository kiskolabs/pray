# OS and registry packaging

Recipes and notes for distributing the Rust `pray` binary and the TypeScript library beyond crates.io, npm, and RubyGems.

Language registries stay under `scripts/release/`. This tree holds OS package recipes and the channel matrix.

## Channel matrix

| Channel | Status | Location | Upload |
|---------|--------|----------|--------|
| jsr.io (`@kiskolabs/pray-core`, `@kiskolabs/pray-cli`) | In-tree release script | `npmjs/pray-core/jsr.json`, `npmjs/pray-cli/jsr.json`, `scripts/release/jsr.sh` | Manual `jsr publish` after scope exists |
| Nix flake | In-tree | Root `flake.nix` | Consumers: `nix run github:kiskolabs/pray` |
| nixpkgs | Planned | Submit from flake derivation | PR to NixOS/nixpkgs |
| AUR (`pray-cli`) | Recipe | `packaging/aur/pray-cli/PKGBUILD` | AUR git after package adoption |
| Debian / Ubuntu | Deferred | `packaging/debian/` | ITP, mentor, or team PPA |
| Fedora / COPR | Deferred | `packaging/fedora/` | COPR project or Fedora package review |
| Guix | Deferred | not started | Guix package definition |
| Flatpak / Snap | Deferred | not started | Low priority for a CLI |
| GitHub Actions CLI archives | In-tree workflow | `.github/workflows/build-cli-release.yml` | Unsigned macOS / Linux / Windows artifacts with `SIGNING` sidecar (`sign_path=local`); create GitHub Release from CHANGELOG when missing, otherwise upload assets onto that release. Operator signs and notarizes locally; Actions holds no signing secrets. |

Homebrew stays in the sibling tap (`make bump-homebrew`), not under `packaging/`.

## Prebuilt CLI archives

The Build CLI release workflow produces:

- `pray-<version>-universal-apple-darwin.tar.gz`
- `pray-<version>-x86_64-unknown-linux-gnu.tar.gz`
- `pray-<version>-x86_64-pc-windows-msvc.zip`

Each archive includes the binary, `VERSION`, `SOURCE_COMMIT`, `SIGNING` (`status=unsigned`, `sign_path=local`, platform, public identity anchor, note), and a per-binary sha256 sidecar. Identity fields come from repository variables: macOS `team_id` (`MACOS_TEAM_ID`), Linux `cosign_identity` (`LINUX_COSIGN_IDENTITY`), Windows `signer_subject` (`WINDOWS_SIGNER_SUBJECT`). Matching `*.sha256` files are uploaded beside the archives.

Unsigned downloads may hit macOS Gatekeeper or Windows SmartScreen. Sign and notarize locally from these CI archives on macOS, Windows, or Linux as the platform tools allow. Production verify policy and `pray.kisko.dev` publish remain tracked in `docs/issues/20261027144200_signed-platform-binaries-on-tag.md`.

## Version source

- Rust binary packages follow the workspace version in root `Cargo.toml`.
- JSR follows `npmjs/pray-core/jsr.json` and `npmjs/pray-cli/jsr.json`, which must match the matching npm package versions. npm and JSR `pray-cli` depend on `pray-core`.

Bump AUR with `scripts/release/aur_bump.sh` after the GitHub tag exists. Nix reads the workspace version from `Cargo.toml` and does not need a separate bump file.
