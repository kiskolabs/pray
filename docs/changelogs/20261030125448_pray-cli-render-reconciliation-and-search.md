# Cross-language render reconciliation and catalog search

## Decisions

Keep plain `pray drift` as the ideal-render comparison defined by the existing render contract. Make `pray render --check`, frozen install, and plan use the same reconciled destination bytes as their corresponding write path.

Remove a retired managed span only when the previous lock tracked it and its current body still matches the previous ideal checksum. Preserve edited retired spans and unknown spans for explicit review.

Use the existing stable `pray search` distribution-index contract in Rust, Ruby, and TypeScript. Do not introduce a second catalog command family or recommendation fields in this change.

## Effects

Rust, Ruby, and TypeScript render checking and writing now agree for preserved unmarked layout and retired spans. Strict verification gives manual removal guidance for unknown markers.

All three CLIs search declared sources, local distribution roots, and registry URLs. Index package names are treated as untrusted input. URL user information is removed from shared HTTP failure messages.

The person-facing surfaces assessed were command help, search results, an empty result, invalid arguments, and orphan-marker guidance. The command hierarchy stays under Inspect; concise help fits a narrow terminal; empty and error states give a next action. Search has no loading or interactive state. A human should still review the exact wording before release.

The final isolated checks completed successfully: the Rust workspace test suite, Clippy with warnings denied, and Rust formatting; 381 Ruby examples and RBS validation; 256 TypeScript core tests and 68 TypeScript CLI tests; TypeScript type checks, Biome checks, and circular-dependency checks; and the repository line-count and diff-whitespace gates. Targeted RuboCop inspection found no offenses after the final edit. The line-count gate reported 137 existing warnings and no unratcheted hard-limit failure.

The repeatable Ruby benchmark records the local 10, 100, and 1,000 package search budgets and the 64 KiB and 1 MiB render-reconciliation budgets. In the final run, median reconciliation time was 0.520 milliseconds at 64 KiB and 9.089 milliseconds at 1 MiB. Median name-only search time was 0.861, 7.760, and 76.121 milliseconds; summary search time was 1.085, 11.752, and 106.537 milliseconds. Summary search retains the sequential index-plus-one-metadata-read-per-match behavior shared by the implementations. The Rust index, summary, render, orphan-scan, resolution, and verification scaling guards also passed.

## Next

Review the unreleased changes and benchmark evidence before release. Remote search latency, transferred bytes, time to first row, and peak memory remain unmeasured; address those before expanding rich catalog browsing.

Keep the TypeScript core and CLI package tests serial until their build outputs are isolated. Concurrent runs can replace the shared core `dist` directory while the CLI suite is reading it; isolated reruns pass.

## Source

- [Render consistency dependency record](../dependencies/20261030124204_pray-cli-render-consistency-and-retired-markers.md)
- [Catalog discovery dependency record](../dependencies/20261030124205_pray-cli-catalog-discovery-gaps.md)
- [RFC 0030](../../rfcs/0030-render-markers-ownership.md)
- [RFC 0040](../../rfcs/0040-cli-surface.md)
- [RFC 0060](../../rfcs/0060-distribution.md)
