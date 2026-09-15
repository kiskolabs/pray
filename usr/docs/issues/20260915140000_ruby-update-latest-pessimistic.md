# Ruby update --latest two-component pessimistic skip

## Participants

Andrei Makarov

## Decisions

Ruby version_satisfies for ~> must use the same conversion as Rust and TypeScript, not Gem::Requirement. Two-component ~> 2.2 admits only the 2.2 line, so pray update --latest rewrites to ~> 2.4 when registry latest is 2.4.0. TypeScript already matches Rust (versionSatisfies("1.5.0", "~> 1.4") is false). No TypeScript change.

Ignore locked versions during Ruby update --latest so a constraint that already admits latest still installs that version. Rust and TypeScript already drop the lock hint when ignore_locked_versions is set.

Stay on the current branch.

## Effects

Ruby Constraint.version_satisfies now uses ruby_pessimistic_to_semver, so ~> 2.2 does not admit 2.4.0. update --latest rewrites that pin to ~> 2.4. ResolveOptions.preferred_lock_version drops the lock pin when ignore_locked_versions is set, so a constraint that already admits latest still installs it.

Confirming checks: bundle exec rspec spec/pray/constraint_spec.rb spec/pray/resolve_context_spec.rb spec/pray/update_latest_spec.rb (14 examples, 0 failures). cargo test -p pray-core constraint (constraint unit tests passed). Isolated TypeScript constraint.test.js: 2 passed, including 2.4.0 vs ~> 2.2.

## Next

None.

## Source

Reported: Ruby 1.15.0 pray update --latest printed All package constraints already allow latest versions for ~> 2.2 with registry latest 2.4.0. Rust 1.14.0 rewrote to ~> 2.4 and installed.
rfcs/0010-core-formats.md
crates/pray-core/src/constraint.rs
npmjs/pray-cli/src/constraint.ts
rubygems/pray-cli/lib/pray/constraint.rb
rubygems/pray-cli/lib/pray/cli/commands/update_latest.rb
