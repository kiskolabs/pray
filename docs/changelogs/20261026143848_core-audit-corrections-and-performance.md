# Core audit corrections and performance

## Participants

Andrei Makarov

## Decisions

Apply the stable Prayfile, prayspec, lockfile, rendering, and distribution contracts consistently across the Rust, Ruby, and TypeScript implementations. Close a finding only with a regression that failed before the correction and passed after it, or with an existing test that exposed the regression.

Treat source containment, frozen and locked execution, provisioned ownership, and public distribution paths as correctness boundaries before optimizing throughput. Keep comparable benchmark inputs and output digests across implementations. Gate the actual Rust binary, Ruby gem, and npm archive sizes independently because their package formats are not directly comparable.

Keep YJIT optional. It improved the repeated Ruby resolve loop but increased the measured cold command time and memory use.

## Effects

- Distribution readers expose only the public layout and reject parent and symbolic-link traversal. Ruby and TypeScript source readers also reject symbolic-link files and package directories.
- Rust merges and revalidates dependency constraints. All three implementations use the documented two-component and three-component pessimistic-version ranges.
- Ruby and TypeScript render all legacy outputs and enforce the target byte ceiling.
- TypeScript locked and frozen installation now require and compare the existing lock. Frozen installation and render check validate current output without entering a write transaction.
- Rust, Ruby, and TypeScript reject multiple provisioned owners of one destination and report both package and export identities.
- Ruby lock and manifest serialization emit valid TOML and JSON for control characters.
- TypeScript rejects covered unknown command arguments as usage errors and redacts URL userinfo in shared HTTP status errors.
- Parser and render indexes remove repeated collection and accumulated-content scans. Shared 1, 10, 100, and 1,000 package fixtures validate the complete render digest.
- Release size checks cover the Rust binary, Ruby gem, and packed and unpacked npm package. Production npm output excludes source maps and test compilation uses a separate directory.
- The settled Rust workspace tests and Clippy checks pass. The TypeScript suite passes 310 tests with its local listener permitted. Focused Ruby source, parser, render, destination, serialization, and conformance checks pass; the full Ruby environment still lacks part of the locked development bundle.

## Next

Port the Rust transitive resolve queue, Ed25519 package trust path, locked-yanked retention, and strict install behavior to TypeScript.

Define the RFC 0104 boundary between discovered and approved federation peers before changing sync traversal. Specify confession identity, replay protection, and retention before treating those records as verified.

Add bounded request and header line readers, complete descriptor-relative no-follow opening where supported, and finish TypeScript command arity validation.

Establish isolated-process variance before making the multi-scale benchmark a blocking CI threshold. Preserve the release size budgets and explain any future relaxation.

## Source

docs/issues/20261026131500_cross-implementation-core-audit.md

docs/issues/20261026114500_source-symlink-rejection.md

rfcs/0010-core-formats.md

rfcs/0011-prayspec-and-package.md

rfcs/0020-resolve-and-lock.md

rfcs/0033-provisioned-destination-safety.md

rfcs/0050-security-and-trust.md

rfcs/0060-distribution.md

rfcs/0104-federation-transports.md
