# Releasing pray

Manual release checklist for the reference CLI and library packages.

Publishing is not automated in CI. Operators run the scripts under `scripts/release/` after tagging and validating.

## Package names

| Surface | Package name | Executable |
|---------|--------------|------------|
| [crates.io](https://crates.io/crates/pray-cli) | `pray-core`, `pray-transport`, `pray-cli` | `pray` (from `pray-cli`) |
| [npm](https://www.npmjs.com/package/pray-cli) | `pray-cli` | `pray` |
| [RubyGems](https://rubygems.org/gems/pray-cli) | `pray-cli` | `pray` |
| pray distribution point | prayers under `prayers/` (see [repository-layouts.md](repository-layouts.md)) | n/a |

The crates.io name `pray` is already taken by an unrelated project, so the Rust CLI publishes as `pray-cli` while keeping the binary name `pray`.

## Version sync

Keep these equal before a full language-registry publish (`make release-all`):

- workspace `Cargo.toml` (`[workspace.package].version`)
- `npmjs/pray-cli/package.json`
- `npmjs/pray-cli/src/lockfile/types.ts` (`PACKAGE_VERSION`)
- `rubygems/pray-cli/lib/pray/version.rb`

The next version crates.io, npm, and RubyGems may share is the max of those numbers. A max held by one surface only cannot be aligned, whether that cut was a patch or a minor. The other surfaces skip that number and jump to the next patch. 1.18.1 is a Ruby-only example; an npm-only 1.19.0 or a crates-only 1.19.0 would likewise be skipped, with the next shared cut at 1.19.1 or later.

`make release-all` still requires all surfaces equal. After a successful full publish it runs `make clean` so `target/` does not keep release and tooling build residue. Calling `scripts/release/all.sh` directly does not clean. `make release-crates`, `make release-npm`, and `make release-rubygems` may publish a one-surface version when that surface is uniquely ahead, print the next coordinated version, and leave the other registries where they are. A shared publish refuses a version that is missing from the root, npm, or Ruby changelog.

## Commands

Dry-run language registries:

```sh
make release-dry-run
# or
./scripts/release/all.sh
```

Publish crates.io (order is handled by the script). A crate whose version is already on crates.io is skipped so `make release-all` can resume:

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

## Man page

The committed man page is `docs/man/pray.1`. `cargo install` does not install it automatically.

From a checkout:

```sh
man ./docs/man/pray.1
# or
install -d "$(manpath | cut -d: -f1)/man1"
install -m 644 docs/man/pray.1 "$(manpath | cut -d: -f1)/man1/pray.1"
```

Exit codes for operators: `docs/cli-exit-codes.md` (normative table in RFC 0040).

## After language registry publish

1. Create a GitHub Release for `vX.Y.Z` so `pray` upgrade notices can resolve the latest tag. Run `make release-github` (or `./scripts/release/github.sh --publish`). The title is `vX.Y.Z`. Notes are that version's `CHANGELOG.md` heading and bullets only. When the annotated tag already exists, the script uses `gh release create --verify-tag` or `gh release edit` and omits `--target`. `--target` accepts a branch or full commit SHA; a short SHA returns HTTP 422. Creating a version other than the workspace version passes `--latest=false`. Preview notes with `./scripts/release/github.sh --notes-only`. Rewrite every existing GitHub Release from `CHANGELOG.md` with `make release-github-sync`.
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
