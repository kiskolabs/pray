# Deprecate skills terminology

## Participants

Andrei Makarov

## Decisions

Deprecate Prayfile keyword skills, export type skill, and prayspec map spec.skills on the same schedule as target, output, and agent. They still parse and warn. They are removed in version 2. Teaching examples use tree or folder. Destination path .agents/skills may stay because that is a host directory name. Internal fields such as ManifestTarget.skills stay.

## Effects

Parsers record skills as a deprecated keyword. Resolve warns once per unique spec.skills or type skill warning. Init and example Prayfiles use folder. RFC 0010, 0011, 0034, 0040, and 0108 stop teaching the old names.

## Next

Remove skills, skill, and spec.skills in version 2. Keep alias tests until then.

## Source

rfcs/0010-core-formats.md
rfcs/0011-prayspec-and-package.md
rfcs/0034-unused-specified-surfaces.md
CHANGELOG.md
