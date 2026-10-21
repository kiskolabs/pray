# Deferred audit security bounds, quiet gate, and recovery tests

## Participants

Andrei Makarov

## Decisions

Ship the deferred engineering-audit follow-ups in the Unreleased train after 1.22.0 rather than holding another registry cut for them alone.

## Effects

Git catalog archive and blob fills share the same size ceilings as praypkg unpack. Ruby and TypeScript unpack through the shared tar validators instead of system tar, including skip of git archive pax global headers. Project catalogs publish through staging, an advisory lock, and rename. Stub trust import accepts only the expected global bare path. Registry and prayspec paths refuse URL-shaped artifacts and escaping members. Global quiet covers apply, install, plan, doctor, and unlock status lines. Doctor redacts URL userinfo. Recovery-hint and doctor tests cover the three reference CLIs.

## Next

Run the second engineering-audit iteration and fold any new high-certainty fixes into Unreleased before the next tag.

## Source

docs/issues/20261021220000_engineering-audit-1-22-release.md
