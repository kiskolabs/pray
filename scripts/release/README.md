# Release scripts

Manual release helpers for language registries and pray distribution points.

Publishing is intentional and operator-driven. Scripts default to dry-run / build-only unless `--publish` is passed.

## Surfaces

| Script | Target | Default | Publish flag |
|--------|--------|---------|--------------|
| `crates.sh` | crates.io (`pray-core`, `pray-transport`, `pray-cli`) | `--dry-run` | `--publish` |
| `npm.sh` | npmjs (`pray-cli`) | pack dry-run | `--publish` |
| `jsr.sh` | jsr.io (`@kiskolabs/pray-cli`) | publish dry-run | `--publish` |
| `rubygems.sh` | RubyGems (`pray-cli`) | build gem | `--publish` |
| `distribution.sh` | pray distribution point | requires `--root` / `--server` | omit `--dry-run` |
| `all.sh` | all of the above | dry-run registries | `--publish` plus DP args |
| `github.sh` | GitHub Release for `vX.Y.Z` | print title and one CHANGELOG section | `--publish` (create or edit); `--all` rewrites existing releases |
| `aur_bump.sh` | `packaging/aur/pray-cli/PKGBUILD` | rewrite version and sha256 | n/a (no upload) |

The CLI executable remains `pray`. The crates.io package name is `pray-cli` because `pray` is already taken by an unrelated crate.

First crates.io publish must go in order: `pray-core`, then `pray-transport`, then `pray-cli`. Until `pray-core` exists on crates.io, `cargo publish --dry-run` for the later crates falls back to `cargo check`. A later `--publish` skips a crate whose version is already on crates.io. The same skip applies to npm (`pray-cli`), jsr.io (`@kiskolabs/pray-cli`), and RubyGems (`pray-cli`) when that version is already published.

## Prerequisites

- Version alignment across `Cargo.toml`, `npmjs/pray-cli/package.json`, `npmjs/pray-cli/jsr.json`, `npmjs/pray-cli/src/lockfile/types.ts`, and `rubygems/pray-cli/lib/pray/version.rb` for `all.sh`. The next coordinated version is the max of crates, npm, and gem. A one-surface max (patch or minor) is skipped by the other registries. Each language script may publish when that surface is uniquely ahead. JSR version must match npm.
- `cargo login` / `CARGO_REGISTRY_TOKEN` for crates.io
- `npm login` for npmjs
- jsr.io: create `@kiskolabs` / `pray-cli` once; local publish uses browser auth; CI may set `JSR_TOKEN`. Provenance requires GitHub Actions OIDC via `.github/workflows/publish-jsr.yml` after the package is linked to this repository in JSR settings. Set package description and runtime compatibility (Node.js, Deno) on the jsr.io Settings tab for search score.
- `gem push` credentials (MFA) for RubyGems
- RubyGems release dependencies are installed under `rubygems/pray-cli/tmp/release-gems/` in a cache keyed by the complete Ruby build and platform. The release helper ignores user-installed gems so native extensions from another Ruby patch release cannot enter the build.
- A working `pray` binary for distribution-point publish (`PRAY` selects it; after `gem push`, PATH `pray` may be the RubyGems CLI)
- `gh` authenticated for `kiskolabs/pray` when creating or editing GitHub Releases
- Optional: `PRAY_RELEASE_YES=1` to skip confirmation prompts
- Optional: `PRAY_SIGNING_KEY` or `--signing-key` for ed25519 package signatures

## Examples

Dry-run all language registries:

```sh
scripts/release/all.sh
```

Publish crates only:

```sh
scripts/release/crates.sh --publish
```

Publish npm, JSR, and RubyGems:

```sh
scripts/release/npm.sh --publish
scripts/release/jsr.sh --publish
scripts/release/rubygems.sh --publish
```

After the GitHub tag exists, refresh the AUR recipe checksum (does not upload to AUR):

```sh
scripts/release/aur_bump.sh
# or
make bump-aur
```

Publish local prayers under `prayers/` to a distribution root:

```sh
scripts/release/distribution.sh --root ./prayers
```

Publish to a remote distribution point:

```sh
scripts/release/distribution.sh --server https://pray.example/registry --signing-key ~/.config/pray/ed25519.seed
```

Full release including distribution point:

```sh
scripts/release/all.sh --publish --root ./prayers --server https://pray.example/registry
```

`make release-all` runs the same publish path, then `make clean` (`cargo clean`) so workspace `target/` does not accumulate. The script alone does not clean.

If a full publish stops after one or more registries succeed, resume with `make release-all` (or the remaining surface scripts). Completed crates.io, npm, jsr.io, and RubyGems versions are skipped automatically.

Create or edit the GitHub Release for the workspace version:

```sh
scripts/release/github.sh --publish
```

Rewrite notes on every GitHub Release that already exists:

```sh
scripts/release/github.sh --publish --all
```

## Makefile targets

From the repository root:

```sh
make release-dry-run
make release-crates
make release-npm
make release-jsr
make release-rubygems
make release-distribution ROOT=./prayers
make release-github
make release-github-sync
make bump-aur
```

`github.sh` reads `CHANGELOG.md`. Title is `vX.Y.Z` so the GitHub name is never empty. Notes stop at the next `##` version heading. `--all` edits releases that already exist; it does not create tags or notify for missing versions.
