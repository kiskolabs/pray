# Releasing pray

Manual release checklist for the reference CLI and library packages.

Publishing is not automated in CI. Operators run the scripts under `scripts/release/` after tagging and validating.

## Package names

| Surface | Package name | Executable |
|---------|--------------|------------|
| crates.io | `pray-core`, `pray-transport`, `pray-cli` | `pray` (from `pray-cli`) |
| npmjs | `pray-cli` | `pray` |
| RubyGems | `pray-cli` | `pray` |
| pray distribution point | packages under `packages/` | n/a |

The crates.io name `pray` is already taken by an unrelated project, so the Rust CLI publishes as `pray-cli` while keeping the binary name `pray`.

## Version sync

Keep these equal before any publish:

- workspace `Cargo.toml` (`[workspace.package].version`)
- `npmjs/pray-cli/package.json`
- `npmjs/pray-cli/src/lockfile/types.ts` (`PACKAGE_VERSION`)
- `rubygems/pray-cli/lib/pray/version.rb`

Release scripts refuse to proceed when these drift.

## Commands

Dry-run language registries:

```sh
make release-dry-run
# or
./scripts/release/all.sh
```

Publish crates.io (order is handled by the script):

```sh
./scripts/release/crates.sh --publish
```

Publish npm and RubyGems:

```sh
./scripts/release/npm.sh --publish
./scripts/release/rubygems.sh --publish
```

Publish local prayer packages to a distribution point:

```sh
./scripts/release/distribution.sh --root ./prayers
./scripts/release/distribution.sh --server https://example.invalid/pray --signing-key ~/.config/pray/ed25519.seed
```

See `scripts/release/README.md` for flags, credentials, and orchestration details.

## After language registry publish

1. Create a GitHub Release for `vX.Y.Z` so `pray` upgrade notices can resolve the latest tag.
2. Optionally bump Homebrew with `make bump-homebrew` once the tag exists.
3. Confirm install paths:

```sh
cargo install pray-cli --locked
npm install -g pray-cli
gem install pray-cli
```

## Registry release feeds

Subscribe in a feed reader to watch publishes land:

| Surface | Feed | Notes |
|---------|------|-------|
| crates.io | https://static.crates.io/rss/crates/pray-cli.xml | Per-crate RSS (also site-wide `crates.xml` / `updates.xml` under the same host) |
| RubyGems | https://rubygems.org/gems/pray-cli/versions.atom | Per-gem Atom (linked as RSS on the gem page) |
| npmjs | — | No first-party per-package RSS/Atom; registry only exposes a global recent-updates feed at `https://registry.npmjs.org/-/rss` |

For npm, poll package metadata (`https://registry.npmjs.org/pray-cli`) or use a third-party feed if you need per-package notifications.
