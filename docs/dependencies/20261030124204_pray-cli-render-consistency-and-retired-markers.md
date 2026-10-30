# pray-cli render consistency and retired markers

## Dependency

pray-cli 1.22.0. Before the unreleased changes recorded in Resolution, the relevant behavior remained unchanged in 1.25.0 and in the implementation inspected on 2026-10-30.

This is a target-scoped behavior report. It is not a full dependency graph, security, or freshness audit.

## Symptom

Two write and verification contracts disagree.

1. `pray render --check` reports a rendered destination as stale when `pray render` would preserve the file byte for byte. Plain `pray drift` can report the same file as different from a fresh render, while `pray drift --semantic` stays clean. The reproduced case differs only by one trailing blank line after the final managed block.
2. A managed block that is no longer tracked remains in the destination after `pray plan`, `pray apply`, and `pray install`. Verification reports an orphan marker and suggests that installation can reconcile it, although installation preserves the block.

The first mismatch creates a false failure for a check whose natural contract is “would this write change the destination?”. The plain drift result is less clear: comparing against an ideal fresh render is internally consistent, but the command should distinguish ideal layout from bytes that the write path intentionally preserves.

The second mismatch is partly a message defect and partly a missing lifecycle distinction. A block that was tracked by the previous lockfile can be retired safely under stronger conditions than an unknown marker that has no ownership record.

## Evidence

The trailing-blank-line case was reproduced against the packaged 1.22.0 implementation with synthetic managed content:

- fresh content ended in one newline;
- existing content ended in two newlines;
- the write-layout operation returned the existing bytes unchanged;
- the write result therefore differed from the fresh render by one byte.

The orphan case was reproduced by adding a second, untracked managed block after a tracked block. The write-layout operation preserved the untracked block.

Implementation inspection confirms the same split:

- checks compare the destination directly with fresh rendered content;
- writes first merge fresh managed content into the existing layout;
- drift normalizes line endings but not end-of-file layout;
- orphan verification says installation can reconcile the marker;
- the merge keeps managed blocks that do not occur in the fresh render.

The report’s command observations are consistent with those contracts: strict verification can stay clean for an extra blank line outside managed content, semantic drift can stay clean, and byte-oriented checks can fail.

The end-to-end command sequence was not repeated against a consumer repository. The independent reproduction covered the packaged merge behavior and the command contracts visible in the implementation.

## Benchmark

The original reproduction recorded a synthetic microbenchmark of the additional cost of using the same managed-content merge for comparison that the write path uses. Each input contained one managed block with roughly 80-byte lines. The ranges below came from four independent benchmark processes on 2026-10-30; each result was a median of seven warm runs:

- 65,553-byte destination: 484.58–525.44 microseconds per comparison;
- 1,048,593-byte destination: 9.79–10.63 milliseconds per comparison.

The current direct comparison rejected the one-byte length mismatch in about 0.03 microseconds in this microbenchmark. That number is only a lower-bound reference: it does not include rendering, file IO, or command startup.

The merge-aware comparison is materially more expensive but remains below one millisecond at the reproduced file size. Cost grows with destination size, so an implementation should avoid parsing the same destination more than once per command. End-to-end command benchmarks are still needed before release.

The repeatable benchmark added with the unreleased implementation measured the full merge comparison once per sample, using roughly 80-byte lines. Seven warm samples in the final run on 2026-10-30 produced these results:

- 65,537-byte destination: 0.520 milliseconds median and 0.803 milliseconds at the 95th percentile;
- 1,048,577-byte destination: 9.089 milliseconds median and 11.558 milliseconds at the 95th percentile.

The benchmark records one managed-content scan per comparison. End-to-end command timing remains a release check.

## Suggested fix

Define and test the command contracts before changing normalization.

1. Make `render --check` compare the on-disk bytes with the bytes that `render` would actually write. Read the destination once, compute the write layout once, and reuse that result.
2. Keep plain `drift` as an ideal-render comparison only if its output explicitly distinguishes ideal layout drift from a pending write. Otherwise, align it with the write layout and leave canonical-layout checking to a separately named mode.
3. Do not delete every unknown marker automatically. Classify a block as retired when the previous lockfile tracked it and the new lockfile does not. Remove it only when its on-disk checksum still matches the previous locked checksum.
4. Treat a marker with no previous ownership record as unknown. Preserve it and tell the operator to inspect and remove it explicitly. Remove the claim that `pray install` reconciles unknown markers.

Add regression coverage before changing behavior:

- `render --check` succeeds exactly when a subsequent render is byte-identical;
- check and write agree for zero, one, and two trailing newlines;
- plain and semantic drift have separately documented expectations;
- an unchanged retired block is removed during reconciliation;
- an edited retired block is preserved and reported;
- an injected unknown block is preserved and receives accurate guidance;
- a large destination is parsed at most once by each check path.

## Next

- Review the unreleased implementation and benchmark evidence before release.

## Resolution

The unreleased Rust, Ruby, and TypeScript implementations now compare `render --check`, frozen install, and plan output with the reconciled bytes the write path would produce. Plain drift retains its existing ideal-render contract.

Reconciliation in all three implementations removes a retired managed span only when the previous lock tracked it and its current managed content still matches the previous ideal checksum. It preserves edited retired spans and unknown spans. An edited active span fails before write. Strict verification now tells the operator to inspect and remove an unknown block explicitly.
