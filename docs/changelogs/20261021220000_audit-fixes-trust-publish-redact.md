# Audit fixes for trust, path-fork delete, and publish redact

## Participants

Andrei Makarov

## Decisions

Ship the fact-checked audit fixes with the Unreleased git-cache and publish-feedback train as 1.22.0.

## Effects

Trust verifies the locked revision before unpacking the catalog. Signed-package policy requires ed25519 and binds fingerprint to public key. Path-fork deletions conflict on upstream change through the apply path. Publish lines redact URL userinfo. Quiet and verbose help names publish scope.

## Next

Publish registries after the coordinated version bump.

## Source

docs/issues/20261021220000_engineering-audit-1-22-release.md
