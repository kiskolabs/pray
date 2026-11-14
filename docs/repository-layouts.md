# Repository layouts

Pray uses `prayers/` for local prayer sources and for a published catalog. Which folders you add depends on whether this project also publishes.

`prayers/v1/packages/` holds catalog metadata. Author prayers in named directories under the path source.

## Consumer

Keep local prayers under a path source. `pray prayer init` defaults to `prayers/<name>/`; an author-chosen path such as `guidance/` remains valid.

```text
Prayfile
prayers/notes/
  notes.prayspec
  exports/notes.md
```

```text
source "local", path: "prayers"
pray "local/notes"
```

The name `v1` is reserved. There is no catalog until you publish.

## Publisher

Keep those sources beside `prayers/v1/`. Run `pray prayer init` and `pray repo init` in the same project. A catalog of many prayers uses the same tree: one named directory per prayer.

```text
Prayfile
prayers/notes/
prayers/prayer-publisher/
prayers/v1/
```

```text
source "local", path: "prayers"
publish "prayers", path: "prayers"
pray "local/notes"
```

A declaration may still use `path:` when the package token is not under a path source, for example `pray "prayer-publisher", path: "prayers/prayer-publisher"`.

Add `spec.version` before `pray package` or `pray publish`. Git install of this repository reads the published catalog under `prayers/v1/`.

Do not add a second top-level source folder. `packages/`, `dist/`, and `shared/` are ordinary names in other tools and are not pray source conventions.

## Standalone package

`pray prayer init` without a Prayfile writes a versioned package in the current directory.

See [static-distribution.md](static-distribution.md) for the files inside `v1/`.
