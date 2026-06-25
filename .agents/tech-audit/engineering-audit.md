# Engineering audit

Pipeline lens and evidence-first finding format for reviews and audits in this repository.

## Pipeline stages

Map findings to stages when relevant:

```
request ingress → app logic → cache → database → queue → worker → external API → response path
```

For Prayfile / pray, typical stages include:

- **resolve** — parse, resolve semver, merge exports, write lockfile
- **render** — fetch, verify hashes, materialize targets
- **verify** — lockfile integrity, cache validity, render consistency
- **drift** — managed blocks differ from lockfile and renderer output
- **CLI egress** — exit codes, diagnostics

## Dimensions

Scan for:

1. **Broken or incomplete behavior** — inconsistent, incomplete, or undefined functionality.
2. **Inadequate test coverage** — missing tests for important paths or contracts.
3. **Futile test coverage** — assertions that only trivially pass, check the wrong thing, or give false confidence. Distinguish explicitly from missing coverage.
4. **Redundancy** — duplicated or superfluous functionality; tangled ownership.
5. **Code quality and organization** — structure that hurts maintenance. Ignore pure style dust unless it harms correctness, operability, or maintainability.
6. **Asymptotic and hot-path shape** — inner loops likely O(n²) where O(n log n) or O(n) is plausible; repeated scans; avoidable allocations; hidden serialization.
7. **Purpose and ownership** — files or modules with no clear role.
8. **Language-native features** — code that fights the language instead of using idioms, stdlib, or platform guarantees.

## Finding format

Each finding must include:

| Field | Content |
|-------|---------|
| Severity | critical, high, medium, low |
| Confidence | high, medium, low |
| Kind | fact or inference |
| Stage | pipeline stage when applicable |
| Evidence | file, line, or observable behavior |
| Fix | smallest credible fix first |

## Ranking

Order findings by:

1. danger
2. certainty
3. impact
4. fix cost

Prefer smallest credible fix before structural rewrite.

Separate missing coverage from futile coverage.

## Output voice

Blunt, compressed, evidence-first. No engagement filler.

Ignore style-only dust unless it harms correctness, operability, maintainability, or auditability under realistic load.
