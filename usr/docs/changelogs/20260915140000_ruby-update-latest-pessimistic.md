# Ruby update --latest two-component pessimistic

## Decisions

Ruby Constraint.version_satisfies converts ~> the same way as Rust and TypeScript. Gem::Requirement treated ~> 2.2 as >= 2.2 and < 3.0, so update --latest skipped the rewrite. The shared conversion treats ~> 2.2 as >= 2.2.0, < 2.3.0.

ResolveOptions.preferred_lock_version returns no lock pin when ignore_locked_versions is set or the package is unlocked, matching Rust PackageResolutionContext and TypeScript lockfilePreferredVersion.

## Effects

Ruby pray update --latest rewrites ~> 1.4 to ~> 1.5 when registry latest is 1.5.0, then locks 1.5.0. A constraint that already admits a newer patch still installs that patch. TypeScript needed no code change.

## Next

None.

## Source

usr/docs/issues/20260915140000_ruby-update-latest-pessimistic.md
rubygems/pray-cli/lib/pray/constraint.rb
rubygems/pray-cli/lib/pray/resolve_context.rb
rubygems/pray-cli/spec/pray/constraint_spec.rb
rubygems/pray-cli/spec/pray/update_latest_spec.rb
