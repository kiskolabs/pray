# Package JSON version growth

Live work. Per-package metadata files under v1/packages grow with every published version. Measured against amkisko/prayers catalog and agentfile layout (RFC 0060). Goal: keep install and update fast and cheap in network, CPU, storage, and energy before any single package JSON reaches megabytes.

## Participants

Andrei Makarov

## Decisions

One file per package that appends every version is the same failure mode as npm packuments: consumers that need only the resolve tip download full history. Do not start with server clustering. Document sharding and thin tip layout are the right first levers; horizontal process clustering does not shrink what each GET carries.

Use soft budgets of 256 KiB and 1 MiB on package metadata documents. Prefer compact JSON write over pretty for published registry files. Keep derived_metadata on the tip (or search surface), not on every historic version row. Integrity, yanked flag, hashes, and signature stay on history rows.

Phase the layout change: Phase 0 is budgets, compact write, tip-only derived fields, and conditional GET (ETag). Phase 1 is RFC 0060 amendment for a thin tip document plus separate history. Phase 2 is version pages or range shards only if history depth still blows the soft budget after tip/history split.

Do not embed version history into v1/index.json. Index stays a name list until search measurements justify summaries there (see the update-and-index-efficiency issue).

Correct the earlier size note in that issue: 400 versions is not about 100 KiB. Mean density on the live prayers catalog is about 968 bytes per version compact and about 1612 bytes pretty, so 400 versions is about 378 KiB compact or about 630 KiB pretty.

## Effects

Measured 2026-10-07 on amkisko/prayers at prayers/v1 (39 packages). Largest package JSON was amkisko/engineering-audit at 28127 bytes pretty and about 17565 bytes compact with 16 versions. Catalog package JSON total about 246 KiB pretty / 148 KiB compact; artifacts about 473 KiB; index.json 1318 bytes; tree du about 1.5 MiB. Mean about 1612 B per version pretty and 968 B compact. derived_metadata was about 49 percent of a compact version row. Tip-only selection for the largest package was about 540 bytes versus full compact about 17613 bytes (about 97 percent unused on install when a lock pins one version).

1 MiB crossing at those means: about 650 versions pretty, about 1080 versions compact, about 2200 versions if history rows keep only integrity-shaped fields (~478 B) without derived_metadata. Weekly publish for five years (~260 versions) stays under 1 MiB. Daily publish for five years (~1825 versions) exceeds 1 MiB on pretty write and approaches or exceeds it on compact full history.

Not urgent for the current prayers catalog (max 28 KiB). Urgent to lock the tip/history contract before any package approaches hundreds of versions so install and update do not keep paying linear history cost.

Server clustering was considered and deferred: it does not reduce bytes per metadata GET, JSON parse cost, client storage, or energy on the hot path.

## Next

Add soft budget warnings or checks on publish when a package document crosses 256 KiB, and fail or require an RFC path at 1 MiB until tip/history lands.

Ship compact write for registry package JSON and index if not already the publish default.

Stop writing derived_metadata onto every appended version; keep it on tip or on a search-facing surface.

Draft RFC 0060 amendment: thin tip (latest non-yanked plus select fields) and history document or pages; install and update fetch tip only; yank and audit fetch history.

Re-measure mean bytes per version after derived_metadata leaves historic rows, then revisit Phase 2 sharding only if needed.

Link this note from docs/issues/20260917152100_update-and-index-efficiency.md so the corrected 400-version size estimate is findable.

## Source

Upstream: docs/issues/20260917152100_update-and-index-efficiency.md, docs/issues/20260627103243_minimal_text_packages_and_derived_metadata.md, rfcs/0060-distribution.md

Catalog measured: amkisko/prayers prayers/v1/packages, prayers/v1/index.json

Code paths: crates publish path that appends versions and writes package metadata; crates/pray-core registry fetch that loads the full package document then selects a version

Commands run: Python walk of package JSON files for size and version counts; wc -c on index.json; du -sh on the prayers tree. Session canvas held the projection chart; durable numbers live in this note.
