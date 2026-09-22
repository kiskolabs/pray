# Conflict policy is fail only

## Participants

Andrei Makarov

## Decisions

RFC 0030 Reference section 45 keeps fail as the only supported render conflict policy. Names warn, append, last_wins, and target_specific stay Unresolved on RFC 0030. RFC 0102 stays destination DSL.

fail checksums on-disk managed marker bodies against lock ideal_checksum on write when a previous lock exists. A human edit inside markers is a render error. Unmarked text is kept. Recovery is pray verify or pray drift, then pray apply. Package-versus-package compose collision is a different concern.

## Effects

TypeScript union, JSON schema, and Ruby parser reject other conflict values, matching Rust.

## Next

An Experimental RFC can specify leftover names only after conflict kinds have algorithms and fixtures.

## Source

RFC 0030. RFC 0031 golden rule 4. RFC 0011 folder exclusivity.
