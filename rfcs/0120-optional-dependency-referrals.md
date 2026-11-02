# RFC 0120: Optional dependency referrals

- Feature Name: optional-dependency-referrals
- Type: Standards Track
- Status: Experimental
- Created: 2026-11-02
- Author: Pray maintainers
- Relates: RFC 0011, RFC 0020, RFC 0040, RFC 0119
- Requires: RFC 0011, RFC 0119

## Summary

A prayspec MAY use `related` for advisory hyperlinks and `add_optional_dependency` for versioned referrals. An optional referral does not select or install its target. When another declaration selects the target, its resolved version MUST satisfy the referral constraint. Optional referrals do not form dependency-cycle edges.

## Motivation

Pray exposes two association shapes whose names do not state their different resolver effects. Treating an optional dependency like npm or Cargo suggests that Pray will attempt installation. Treating it like `related` discards its package-name and version-constraint contract. The current reference implementations also reject mutual optional declarations as dependency cycles while ignoring their version constraints.

## Guide-level explanation

Use `related` when a person should discover another prayer without asserting package identity or version compatibility:

```manifest
spec.related = ["sample/testing", "../security"]
```

Use an optional dependency when the target is a named package and its version matters only if the project already selected it:

```manifest
spec.add_optional_dependency "sample/testing", "~> 2.0"
```

The declaration above does not install `sample/testing`. If the Prayfile or a required dependency selects `sample/testing`, resolution fails unless its resolved version satisfies `~> 2.0`.

`pray explain` separates the three roles:

```text
dependencies: sample/base
optional referrals: sample/testing (~> 2.0)
related prayers: ../security
```

`pray tree` shows required dependency edges only.

## Reference-level explanation

### Association roles

`related` retains the advisory, no-install, no-trust contract in RFC 0119. It accepts package shorthand, source locators, and absolute URIs. It carries no version constraint.

`add_dependency` declares a required package edge. Resolution selects the target, applies its constraint, and includes the edge in dependency-cycle detection and `pray tree`.

`add_optional_dependency` declares an optional referral with a package name and version constraint. Resolution MUST NOT select, fetch, install, or render a package solely because of this declaration.

### Selected-target validation

After constructing the resolved package set, an implementation MUST examine every optional referral whose package name is present in that set. The selected target version MUST satisfy the referral constraint. A mismatch MUST fail resolution and identify the declaring package, target package, and failed constraint.

An optional referral whose target is absent from the resolved package set has no resolver effect. A required edge or Prayfile declaration can select the target.

### Graph behavior

Dependency-cycle detection and `pray tree` MUST use required dependency edges only. Mutual optional referrals are valid. A cycle containing required edges remains an error even when optional referrals also connect the packages.

For lockfile compatibility, a package record's `dependencies` name list continues to include required and optional declarations. The prayspec remains authoritative for the declaration kind and constraint. A lockfile consumer MUST NOT infer installation or graph behavior from that name list alone.

### Explanation

`pray explain` MUST print required dependency names under `dependencies`, optional package names and constraints under `optional referrals`, and RFC 0119 references under `related prayers`. An empty group prints `none`.

## Security considerations

An absent optional target causes no lookup or network request. A selected target passes through normal source, integrity, and trust checks before optional-referral validation. A referral does not grant trust or authorize a source.

## Registrar

This RFC claims the `pray explain` label `optional referrals`.

## Drawbacks

The method name remains easy to confuse with package managers that conditionally install optional dependencies. The lockfile dependency name list does not preserve the declaration kind or constraint, so consumers that need those details must inspect the package specification.

## Rationale and alternatives

Folding optional dependencies into `related` was rejected because advisory locators do not assert a package identity or version range. Giving optional dependencies npm or Cargo installation behavior was rejected because Pray has no feature or host-selection mechanism for activating them. Retaining optional edges in cycle detection was rejected because an edge that does not select its target cannot create an installation-order cycle.

Removing `add_optional_dependency` was rejected because a selected peer can still need a compatibility assertion. Adding peer, feature, or plugin-host machinery was deferred until a concrete consumer needs activation semantics.

## Prior art

RubyGems distinguishes runtime and development dependencies; Bundler optional groups are application-level installation groups. npm optional dependencies are attempted installs whose failure can be tolerated, while optional peer dependencies describe a host contract without automatic installation. Cargo optional dependencies are activated through features. Pray adopts none of those activation contracts.

## Unresolved questions

Whether a later lockfile format should preserve required and optional dependency declarations separately.

## Future possibilities

A version 2 prayspec may rename the method to state the referral role directly. A later RFC may add activation semantics for a demonstrated plugin-host or feature consumer.
