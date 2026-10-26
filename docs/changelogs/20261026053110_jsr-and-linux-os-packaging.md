# JSR and Linux OS packaging

## Decisions

Add jsr.io as a language-registry surface for the TypeScript library. Add in-tree Nix and AUR recipes for the Rust binary. Keep Debian and Fedora documented but unpublished until a maintainer path exists.

## Effects

- npmjs/pray-cli/jsr.json and scripts/release/jsr.sh land; all.sh and make release-jsr call them.
- Root flake.nix builds pray-cli. packaging/aur/pray-cli/PKGBUILD and scripts/release/aur_bump.sh land. packaging/debian and packaging/fedora hold deferred notes.
- docs/releasing.md, scripts/release/README.md, packaging/README.md, and README install sections describe the new surfaces.

## Next

Create the jsr.io scope and package, then publish. Adopt the AUR package. Prove the flake on a Nix host and open nixpkgs when ready.

## Source

docs/issues/20261026053110_jsr-and-linux-os-packaging.md
