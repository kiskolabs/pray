# Pray symbol templating

## Participants

Andrei Makarov

## Decisions

- Use `((pray:path))` placeholders with a strict no-space grammar and colon between resolver and path.
- Declare project-wide symbols once with `pray do … end` (alias `template do`), applied to all compose, local, and UTF-8 provisioned renders.
- Only implement the `pray` resolver in this release; leave room for later resolvers without changing placeholder syntax.

## Effects

- pray-core and TypeScript pray-cli substitute symbols at render time and verify exclusive file bindings against substituted content.
- SPEC and schema document the `symbols` map and placeholder rules.

## Next

- Publish a pray CLI release that includes templating before consumers can `pray update` to community packages that ship placeholders.
- Publish amkisko/prayers community-security and community-code-of-conduct 1.1.0 with placeholders once the CLI is available.

## Source

- SPEC.md section "pray symbols (templating)"
- CHANGELOG.md Unreleased
