# JSR and Linux OS packaging

## Participants

Andrei Makarov

## Decisions

- Add jsr.io as a language-registry publish surface for the TypeScript library in `npmjs/pray-cli`, scoped as `@kiskolabs/pray-cli`. The npm package remains the Node global CLI install path; JSR publishes TypeScript source for Deno and other JSR consumers.
- Keep OS packaging recipes in-tree under `packaging/` and a root `flake.nix` for Nix. First concrete Linux targets are AUR (`pray-cli` source package) and Nix (flake plus later nixpkgs PR). Debian, Fedora or COPR, and other channels stay documented until a maintainer path exists.
- Release helpers stay manual and dry-run by default, matching crates.io, npm, and RubyGems. First real JSR publish needs a created `@kiskolabs` scope and package on jsr.io, then `scripts/release/jsr.sh --publish`.
- Do not automate AUR or nixpkgs uploads from this repository. Operators bump local recipes; AUR and nixpkgs land through their own maintainer workflows.

## Effects

- `npmjs/pray-cli/jsr.json`, `scripts/release/jsr.sh`, and orchestration hooks land with version sync against npm.
- Root `flake.nix` builds `pray-cli` from this workspace. `packaging/aur/pray-cli/PKGBUILD` builds the same binary from the GitHub tag tarball.
- `packaging/README.md`, `docs/releasing.md`, `scripts/release/README.md`, and README install sections describe the new surfaces and the deferred Linux channels.
- Later pass 20261026155200: public API JSDoc and no-slow-types dry-run; CI dry-run gate; `.github/workflows/publish-jsr.yml` for OIDC provenance. Description and runtime compatibility still set on jsr.io Settings.

## Next

- On jsr.io package Settings: set description (from npm package description), mark Node.js and Deno compatible (Bun optional), link the GitHub repository `kiskolabs/pray`.
- Republish after symbol docs and slow-type fixes so the score reflects them; prefer `.github/workflows/publish-jsr.yml` for OIDC provenance (token publish does not create provenance).
- Submit or adopt an AUR package from `packaging/aur/pray-cli`.
- Open a nixpkgs PR once the flake derivation is proven on a Nix host (`nix build`, `nix run`).
- When wanted: Debian or Ubuntu ITP or PPA, Fedora COPR or rpm, Guix; Flatpak and Snap stay low priority for a CLI binary.

## Source

- docs/releasing.md
- scripts/release/README.md
- packaging/README.md
- https://jsr.io/docs/publishing-packages
