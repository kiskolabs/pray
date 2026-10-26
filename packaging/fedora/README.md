# Fedora and COPR packaging

Not published yet. Prefer an rpm that builds `pray-cli` from the tagged source archive.

## Suggested shape

- Package name: `pray-cli`
- Provides `/usr/bin/pray`
- BuildRequires: rust, cargo, and the usual devel packages for a crates.io graph
- Source: GitHub `vX.Y.Z` archive or crates.io crate tarball for `pray-cli` plus path workspace members from the same tag

## Paths to land

1. Create a COPR project that builds from this repository's tags, or
2. Open a Fedora package review when a long-term maintainer is ready

Keep Fedora uploads off the language-registry release scripts. Bump after the GitHub Release exists.
