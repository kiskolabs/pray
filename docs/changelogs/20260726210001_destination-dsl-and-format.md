# Destination DSL and pray format

## Participants

Andrei Makarov and Cursor.

## Decisions

Prayfile gains recommended destination forms on prayfile "1": compose, tree, and pray with file:. Legacy target, agent, package, output, skills, and local remain valid.

Default export selection uses destination context: fragment in compose, folder or skill in tree, file for file:. Legacy-only manifests keep selecting all exports.

pray format (alias fmt) rewrites Prayfile to the recommended destination DSL. It classifies packages from resolved export kinds, offline first. Marker normalization in lockfile outputs remains a secondary step.

## Effects

Added destination binding helpers, format serializer, schema fields for destinations and file exports, and SPEC sections for compose, tree, pray, and format.

Validated with cargo test -p pray-core and cargo test -p pray --test cli_ux.

## Next

Consumers can migrate with pray format after install has resolved packages.

## Source

Local main worktree; destination DSL plan and format command follow-up.
