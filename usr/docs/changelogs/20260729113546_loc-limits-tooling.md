# LOC limits tooling

## Participants

- Andrei Makarov

## Decisions

- Soft warn at 150 lines and hard fail above 300 for production sources under crates/*/src, rubygems/pray-cli/lib, and npmjs/pray-cli/src.
- Shared check is scripts/check-loc-limits.sh with ratchet list scripts/loc-limits.allowlist for current overages.
- Ruby uses custom RuboCop Pray/FileLength at Max 300; TypeScript uses Biome noExcessiveLinesPerFile at maxLines 300; Rust has no Clippy file-LOC lint so relies on the shared script.

## Effects

- Added make loc-check and CI step on the rust job.
- Grandfathered existing files above 300 via allowlist ratchet and matching RuboCop Exclude / Biome overrides.

## Next

- Split allowlisted files over time and remove ratchet rows when each file is at or under 300.
- Watch Clippy issue for too_many_lines_in_file and drop Rust reliance on the script when that lands.

## Source

- AGENTS.md file size guidance (150 soft / 300 hard)
