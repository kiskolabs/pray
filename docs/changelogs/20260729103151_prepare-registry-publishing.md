# Prepare registry publishing

## Participants

- Andrei Makarov

## Decisions

- Publish the Rust CLI on crates.io as pray-cli because the crate name pray is already taken by an unrelated project; keep the installed binary name pray.
- Publish pray-core and pray-transport as well so path dependencies can resolve from crates.io.
- Keep npmjs and RubyGems package names as pray-cli.
- Release publishing stays manual; scripts under scripts/release default to dry-run or build-only and require an explicit --publish for registry push.
- Distribution-point release uses a temporary publisher Prayfile that only declares packages under packages/, then runs pray package and pray publish.

## Effects

- Renamed the Cargo package from pray to pray-cli with an explicit [[bin]] name pray.
- Added workspace.dependencies version+path wiring for publishable crates.
- Added crate READMEs and crates.io metadata.
- Fixed TypeScript PACKAGE_VERSION drift (was 1.2.0, package.json is 1.5.0).
- Added scripts/release/{common,crates,npm,rubygems,distribution,all}.sh and Makefile release targets.
- Documented the operator flow in docs/releasing.md and scripts/release/README.md.
- Validated: cargo test -p pray-cli --test version --test cli_upgrade passed; cargo publish -p pray-core --dry-run --allow-dirty packaged successfully; pray-transport and pray-cli dry-run correctly require pray-core on crates.io first; distribution.sh --dry-run packaged prayer-publisher; gem build produced pkg/pray-cli-1.5.0.gem.
- Reworked registry descriptions and README openers so Rust, npm, and Ruby share the same Prayfile positioning line.

## Next

- Operator runs scripts/release/crates.sh --publish, npm.sh --publish, rubygems.sh --publish after credentials and tag are ready.
- Operator runs scripts/release/distribution.sh with --root and/or --server for prayer packages.
- Create a GitHub Release for the version tag so upgrade notices resolve.
- Confirm post-publish install: cargo install pray-cli, npm install -g pray-cli, gem install pray-cli.

## Source

- Branch: patch/prepare-registry-publishing
- Docs: docs/releasing.md, scripts/release/README.md
