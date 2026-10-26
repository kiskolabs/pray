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

Homebrew stays in the sibling tap (`make bump-homebrew`), not under `packaging/`.

## Version source

- Rust binary packages follow the workspace version in root `Cargo.toml`.
- JSR follows `npmjs/pray-core/jsr.json` and `npmjs/pray-cli/jsr.json`, which must match the matching npm package versions. npm and JSR `pray-cli` depend on `pray-core`.

Bump AUR with `scripts/release/aur_bump.sh` after the GitHub tag exists. Nix reads the workspace version from `Cargo.toml` and does not need a separate bump file.
