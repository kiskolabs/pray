# pray-cli catalog discovery gaps

## Dependency

pray-cli 1.22.0. Before the unreleased changes recorded in Resolution, the relevant command, package-spec, metadata, and federation paths remained unchanged in 1.25.0 and in the implementation inspected on 2026-10-30.

This note describes a product capability gap, not a runtime defect or a full dependency audit.

## Symptom

The CLI can inspect packages already declared by a consumer, but it has no built-in command for browsing every package in a distribution catalog, listing recent catalog changes, or presenting soft package recommendations.

The gap has three separate parts:

1. `pray list` lists resolved packages from the current project. The command parser has no `search`, `catalog`, `browse`, or `index` command for undeclared packages.
2. `pray outdated` evaluates packages already in the project. The CLI has no human-facing recent-release view or stable machine feed for a catalog.
3. Package specifications express required and optional resolution dependencies, but not non-binding relationships such as “consider this companion package”. The CLI therefore cannot distinguish a hard dependency from editorial guidance.

These are feature requests with different contract costs. Catalog browsing can be added on top of the existing distribution format. A first-class recommendation field changes the package contract and should begin as an RFC.

## Evidence

The 1.22.0 command implementation shows the following behavior:

- `list` iterates the resolved project packages;
- `tree` and `explain` also start from packages resolved for the project;
- `outdated` compares project resolution and lock state;
- the parser exposes no catalog browse, recent-release, or recommendation verb.

The distribution data already supports a limited browse path:

- the catalog index contains package names;
- per-package version records can contain publication time, export information, and derived metadata;
- federation index rows contain a package name and update time;
- discovery advertises a `since` parameter name, but the synchronization client fetches the full index without sending that parameter.

The package-spec parser recognizes required and optional dependencies. It has no `related`, `recommends`, `suggests`, or `see_also` field. Derived metadata includes summaries, topics, categories, possible effects, and a hash-bucket embedding, but no consumer command queries those values.

Derived metadata is computed guidance, not verified package intent. It can support search hints, but it should not silently become an authoritative recommendation graph.

No distribution instance was inspected for this revision. The factual claims are limited to the packaged CLI behavior and data contracts.

## Benchmark

The current formats establish a request budget that implementation must address:

- listing names requires one catalog-index request;
- adding latest version, summary, exports, or publication time requires up to one additional metadata request per package with the current static index, for a total of `1 + N` requests for `N` packages;
- the federation index can list names and update times in one request, but not the richer package details proposed for browsing.

Before expanding rich listing, record cold- and warm-cache baselines for 10, 100, and 1,000 packages. Measure request count, bytes transferred, time to first row, total duration, and peak memory. A design that promises rich listing should either paginate, bound concurrency, or add summary fields to a versioned index contract instead of hiding an unbounded `1 + N` request pattern.

The repeatable local-distribution benchmark added with the unreleased implementation ran seven warm samples in the final run on 2026-10-30. Median and 95th-percentile durations were:

- 10 packages: 0.861 and 1.999 milliseconds for names, 1.085 and 1.280 milliseconds with summaries;
- 100 packages: 7.760 and 8.183 milliseconds for names, 11.752 and 15.476 milliseconds with summaries;
- 1,000 packages: 76.121 and 77.745 milliseconds for names, 106.537 and 119.402 milliseconds with summaries.

The names path reads one index file. The summary path reads the index plus one metadata file per matched package: 11, 101, and 1,001 reads in these fixtures. This local measurement does not establish remote latency, transferred bytes, time to first row, or peak memory. The HTTP path retains the same sequential `1 + N` request shape as the Rust reference implementation.

## Suggested fix

Start with the smallest capability that uses the existing contract.

1. Add `pray catalog list --source NAME` with a stable JSON form. The first version can list package names from the catalog index without downloading every package document.
2. Add `pray catalog show PACKAGE --source NAME` for version, export, publication, and derived-metadata details. Label derived fields as computed claims.
3. Add a client-side recent view only after defining timestamp and watermark behavior. Do not advertise server-side `since` filtering until the server consumes and tests the parameter end to end.
4. Benchmark the two commands before deciding whether the catalog index needs versioned summary rows, pagination, or bounded parallel metadata fetches.
5. Handle soft recommendations in a separate RFC. Define their semantics, reason text, destination hint, trust model, cycle behavior, and whether distributors may add them independently of package authors.

Do not mine prose references into recommendations in the first implementation. That adds an unverified inference path before the explicit contract exists.

Acceptance coverage should include an empty catalog, one package, pagination or configured limits, missing optional metadata, malformed timestamps, offline cache behavior, unavailable package metadata, deterministic JSON, and clear separation between hard dependencies and advisory relationships.

## Next

- Review the unreleased cross-language search implementation and benchmark evidence before release.
- Keep recent-release views, rich catalog details, and first-class recommendations out of this change. Start an RFC if recommendation semantics are accepted as a package contract change.

## Resolution

The stable CLI and distribution RFCs already define `pray search <query>` as a substring match over a distribution index. The unreleased Rust, Ruby, and TypeScript implementations now follow that contract instead of introducing a second `catalog` command family.

Search accepts a declared source, a local distribution root, or a registry URL. It validates the two-segment registry identity from the untrusted index, uses deterministic lexical ordering, selects the latest non-yanked summary when present, and identifies summaries in help as unverified catalog claims. Rich results still use the existing `1 + N` metadata-read shape; the accompanying benchmark makes that budget explicit.
