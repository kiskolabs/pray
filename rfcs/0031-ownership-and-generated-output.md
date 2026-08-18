# RFC 0031: Ownership and generated-output contract

- Feature Name: ownership-and-generated-output
- Type: Standards Track
- Status: Stable
- Created: 2026-08-18
- Author: Andrei Makarov
- Relates: RFC 0030, RFC 0010, RFC 0102
- Requires: RFC 0030

## Summary

Three ownership zones separate recipe, human-owned embeds, and managed output. Pray regenerates managed spans from lock, packages, and listed local files.

## Motivation

If humans and tools edit the same generated file, lock and render cannot be reconstructed.

## Guide-level explanation

Recipe zone: Prayfile, packages, lock; humans edit via CLI. `.agents` zone: human-owned embeds; pray reads and does not overwrite. Managed zone: generated roots and package-owned skills; pray regenerates.

## Reference-level explanation

This section is the product contract for this concern. Where it disagrees with Implementation notes, Implementation notes record what the reference CLI does today. A follow-on RFC records the gap.

### 80. Ownership and generated-output contract

The hardest part of Prayfile is keeping managed rendered output stable and read-only while local additions remain editable and safe from overwrite.

The model is not one shared rendered file everyone edits. It is three zones with different owners.

#### Three zones

- Recipe: source Prayfile, packages, Prayfile.lock; edits Humans via `pray add`, `pray remove`, `pray update`; pray resolves and locks
- `.agents`: source `.agents/` (human-owned; `.agents/skills/` is package-managed); edits Humans and applications; pray reads on render; never writes
- Managed: source `AGENTS.md`, generated target files, package-owned rules; edits Nobody directly; pray fully regenerates from lock + recipe + `.agents` inputs

Package exports live only in the managed zone. They are pinned by recipe and hash. Applications consume them; they do not rewrite them.

Human-owned files under `.agents/` (outside package-managed `.agents/skills/**`) are not locked, not hashed in `Prayfile.lock`, and are re-embedded into rendered output on every `pray render` when listed in `Prayfile`.

#### Golden rules

1. Applications must not edit managed files or managed blocks.
2. Applications may edit human-owned files under `.agents/` when project-specific input must change.
3. Humans change shared packages by editing Prayfile and running pray, not by patching rendered target files.
4. Render reconstructs managed output from inputs. There is no three-way merge in v1.

#### Render composition

Root files are assembled in a fixed order:

```
preamble              # short contract (generated)
embedded inputs       # files listed in Prayfile under `.agents/`
managed blocks        # one block per package export
index                 # names only; bodies live elsewhere
```

Managed blocks use opaque pray markers from section 41:

```md
<!-- pray:p7f3k9m2 -->

...rendered content...

<!-- pray:p7f3k9m2 -->
```

On render, pray replaces each managed block from locked package content and re-embeds listed `.agents` files into their spans. Anything outside allowed marker regions is a verify error.

#### Target preamble

Every generated root file may start with a short, user-facing contract. It must not mention implementation details.

Recommended shape:

```markdown
<!-- pray:0 ignore-comments -->

## Input context

Do not edit managed blocks in `AGENTS.md` or skills under `.agents/`.
To change shared guidance, update `Prayfile` and run `pray`.
```

The ignore marker is for tooling. The visible lines are for the application.

#### Managed output ownership

Managed output installs under the target directory for the current project.

Each managed directory or file must carry origin metadata, either in front matter or a small `.pray-origin.toml`:

```toml
package = "sample/webapp"
export = "code-review"
version = "2.1.5"
tree_hash = "sha256:..."
```

Optional human-owned files under `.agents/` are not origin-tagged as packages. Name collisions between human-owned and managed content are conflicts unless policy says otherwise.

Applications must not edit managed directories. They may edit other files under `.agents/`.

#### Idempotency

Definition: same inputs must yield the same managed bytes.

Inputs to render:

- Prayfile.lock
- resolved package trees (verified by tree hash)
- `.agents/**` contents listed in `Prayfile`
- render policy from Prayfile
- target adapter

Guarantees:

- `pray install`: lock and inputs unchanged → no writes (optional optimization)
- `pray render`: always reconstructs managed zones from inputs
- `pray install --frozen`: fails instead of mutating when output would change
- render phase (internal): same lock + same local files + same packages → byte-identical managed output

Local file edits change only local embeds and the root file hash. They do not require resolve unless Prayfile changed.

Package updates change only managed blocks owned by affected packages.

#### Update behavior

```
pray update sample/webapp
```

1. resolve selects new version within constraints and updates Prayfile.lock.
2. render replaces every managed block mapped to `sample/webapp` in `Prayfile.lock`.
3. render replaces managed directories whose origin package is `sample/webapp`.
4. Embedded `.agents` files are re-read but not modified on disk.
5. `pray drift` shows recipe, lock, managed-block, and render changes.

Pray markers make updates surgical in diffs even though render is logically full reconstruction.

#### Remove behavior

```
pray remove sample/webapp
```

1. Remove declaration from Prayfile.
2. resolve recomputes lock without that package.
3. render deletes all managed blocks mapped to `sample/webapp`.
4. render deletes managed directories tagged with that package origin.
5. Human-owned `.agents/**` files are preserved.
6. Orphan pray markers after remove are verify errors.

#### Verify enforcement

See sections 32.2, 52, and 53. Applications are untrusted editors: managed output is like compiled output. Rewrite of a managed block fails the next render or frozen CI check; fix by regenerating and moving custom text into `.agents/` or updating Prayfile.

```
Prayfile + packages  →  resolve  →  lock
lock + local + packages  →  render  →  rendered targets
```

---

## Implementation notes

`render_compose`, `render_write`, `render_provisioned`. Group membership filters which spans render for a selected environment; lock still lists all packages (RFC 0020).

## Unresolved questions

Marker id generation stability across equivalent renders (reserved RFC 0110).
