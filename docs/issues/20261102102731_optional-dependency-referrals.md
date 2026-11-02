# Specify optional dependency referrals separately from advisory links

## Participants

Andrei Makarov (@amkisko)

## Decisions

Keep both prayspec association APIs. `related` remains advisory discovery metadata under RFC 0119. `add_optional_dependency` becomes a versioned package referral: it does not select a target, but a target selected by another declaration must satisfy its constraint.

Exclude optional referrals from dependency-cycle detection and dependency-tree output. Show them with constraints under `optional referrals` in `pray explain`. Keep required package edges under `dependencies` and advisory references under `related prayers`.

RFC 0120 records the contract. Peer, feature, and plugin-host activation remain outside this change.

## Effects

Before the correction, mutual optional declarations failed with `dependency cycle detected` in Rust, Ruby, and TypeScript. A selected target outside an optional constraint resolved successfully because no implementation checked the constraint. The new regressions failed on both behaviors before the implementation and pass after it.

The Rust, Ruby, and TypeScript implementations now accept mutual optional referrals, validate a selected target, and leave an absent target unselected. Required dependency cycles still fail.

The dependent-field audit found one additional TypeScript tree renderer after the first correction. Its regression showed that `pray tree` still nested an optional referral as a dependency edge. The renderer now omits that edge like the Rust and Ruby CLIs.

The fact check supports the external comparison with narrower wording. RubyGems gemspecs expose runtime and development dependencies, while Bundler optional groups are application-level groups. npm optional dependencies are attempted installs whose failure can be tolerated, and optional peers are not installed automatically. Cargo optional dependencies are activated by features. None matches Pray's referral contract exactly. A community discussion about documentation-only Cargo dependencies is not normative evidence and is not needed for the design.

The named Cargo binary reports `pray 1.26.0`, and this repository's lockfile records the same generator version. The shell shim still reports 1.22.0, so command evidence must name the executable. The catalog package paths listed in the source report are not present in this repository; their usage counts remain downstream testimony rather than verified facts in this change.

Validation passed with `cargo test --offline -q`, `cargo clippy --offline --workspace --all-targets -- -D warnings`, and `cargo fmt --all -- --check`. Ruby passed 389 examples with `bundle exec rspec --format progress` and the changed files passed Standard Ruby. TypeScript core passed 263 tests, TypeScript CLI passed 70 tests, and both packages passed Biome and circular-dependency checks. `make loc-check` reported 137 existing soft warnings and 0 hard-limit failures. `git diff --check` passed.

## Next

- Review RFC 0120 before advancing it from Experimental.
- Decide in a later RFC whether lockfiles should preserve dependency kinds and constraints separately.

## Source

- [RFC 0120](../../rfcs/0120-optional-dependency-referrals.md)
- [RFC 0119](../../rfcs/0119-related-prayers.md)
- [RubyGems specification reference](https://guides.rubygems.org/specification-reference/)
- [Bundler groups](https://bundler.io/guides/groups.html)
- [npm package.json](https://docs.npmjs.com/cli/configuring-npm/package-json/)
- [Cargo features](https://doc.rust-lang.org/cargo/reference/features.html)
- `crates/pray-core/tests/optional_dependency.rs`
- `rubygems/pray-cli/spec/pray/optional_dependencies_spec.rb`
- `npmjs/pray-core/src/resolve/optional-dependency.test.ts`
