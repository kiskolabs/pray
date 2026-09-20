# Repository layout note for product publishers

## Participants

Andrei Makarov

## Decisions

Keep prayer sources under prayers/. Consumer trees use prayers/<name>/. A publisher keeps those sources beside prayers/v1/. A catalog of many prayers uses the same tree.

prayers/v1/packages/ is catalog metadata. Root packages/, dist/, and shared/ are not pray source conventions. dist/ already means built output in other tools and already names the published catalog dest in pray.

A later pass on 2026-09-20 dropped the catalog-only packages/ source shape. That folder was this repository's release-script convention, not a spec. RFC 0117 already rejected putting local prayers under packages/.

CLI help for prayer and repo says keep sources under prayers/<name>/. docs/repository-layouts.md is the human layout page. Implementations must not require a root packages/ directory to publish a path-owned package.

A product repository such as scout-cli keeps a one-line pointer at its prayer README. The convention lives in pray.

## Effects

This repository moved packages/prayer-publisher to prayers/prayer-publisher, beside prayers/v1/. Examples use prayers/ for path sources. scripts/release/distribution.sh publishes path-owned packages from the project Prayfile.

Person-facing surface: pray help prayer and pray help repo, plus docs/repository-layouts.md. Hierarchy is consumer versus publisher under one prayers/ tree. Copy is folder names and init commands. Interactive states and empty or error screens do not apply. Remaining for a human: read the help text on a terminal.

## Next

Downstream product repos can point at docs/repository-layouts.md.

## Source

rfcs/0117-local-prayers.md
rfcs/0002-problem-and-positioning.md
docs/repository-layouts.md
crates/pray-cli/src/help_text.rs
scripts/release/distribution.sh
