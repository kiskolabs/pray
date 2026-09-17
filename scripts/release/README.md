# Release scripts

Manual release helpers for language registries and pray distribution points.

Publishing is intentional and operator-driven. Scripts default to dry-run / build-only unless `--publish` is passed.

## Surfaces

| Script | Target | Default | Publish flag |
|--------|--------|---------|--------------|
| `crates.sh` | crates.io (`pray-core`, `pray-transport`, `pray-cli`) | `--dry-run` | `--publish` |
| `npm.sh` | npmjs (`pray-cli`) | pack dry-run | `--publish` |
| `rubygems.sh` | RubyGems (`pray-cli`) | build gem | `--publish` |
| `distribution.sh` | pray distribution point | requires `--root` / `--server` | omit `--dry-run` |
| `all.sh` | all of the above | dry-run registries | `--publish` plus DP args |
| `github.sh` | GitHub Release for `vX.Y.Z` | print title and one CHANGELOG section | `--publish` (create or edit); `--all` rewrites existing releases |

The CLI executable remains `pray`. The crates.io package name is `pray-cli` because `pray` is already taken by an unrelated crate.

First crates.io publish must go in order: `pray-core`, then `pray-transport`, then `pray-cli`. Until `pray-core` exists on crates.io, `cargo publish --dry-run` for the later crates falls back to `cargo check`.

## Prerequisites

- Version alignment across `Cargo.toml`, `npmjs/pray-cli/package.json`, `npmjs/pray-cli/src/lockfile/types.ts`, and `rubygems/pray-cli/lib/pray/version.rb` for crates, npm, and `all.sh`. `rubygems.sh` may publish a gem-only patch when `Pray::VERSION` is ahead of the workspace.
- `cargo login` / `CARGO_REGISTRY_TOKEN` for crates.io
- `npm login` for npmjs
- `gem push` credentials (MFA) for RubyGems
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

Publish npm and RubyGems:

```sh
scripts/release/npm.sh --publish
scripts/release/rubygems.sh --publish
```

Publish local packages under `packages/` to a distribution root:

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
make release-rubygems
make release-distribution ROOT=./prayers
make release-github
make release-github-sync
```

`github.sh` reads `CHANGELOG.md`. Title is `vX.Y.Z` so the GitHub name is never empty. Notes stop at the next `##` version heading. `--all` edits releases that already exist; it does not create tags or notify for missing versions.
