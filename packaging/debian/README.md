# Debian and Ubuntu packaging

Not published yet. Prefer packaging the Rust binary built from the tagged source archive, same layout as the AUR recipe.

## Suggested shape

- Source package name: `pray-cli`
- Binary package: `pray` or `pray-cli` installing `/usr/bin/pray`
- Build with `cargo` from the release tarball and `Cargo.lock`
- Ship `docs/man/pray.1` and the MIT license

## Paths to land

1. File an Intent to Package (ITP) and find a Debian mentor, or
2. Host a personal or team PPA that builds from GitHub tags

Do not upload from the language-registry release scripts. Keep Debian uploads on a maintainer checklist after crates.io and the GitHub Release exist.
