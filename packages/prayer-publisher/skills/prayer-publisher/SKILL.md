---
name: prayer-publisher
description: Turn source text, files, or folders into packaged prayer and publish it.
---

# Prayer Publisher

## Purpose

Turn a source text file, folder, or existing skill into a packaged prayer and publish it through the Pray workflow.

## When to use

Use this skill when you want to:

- package existing guidance as prayer-managed input
- convert one file or many files into a skill directory
- install the result under `.agents/skills`
- build a `.praypkg` and publish it to a distribution point

## Inputs

Preferred inputs:

- a single Markdown or text file
- a folder of Markdown/text files
- an existing skill directory with `SKILL.md`
- the target skill name
- the desired package name and version

## Process

1. Identify the source root.
   - If the input is a folder, decide which files belong in the package.
   - Keep the source static and deterministic.

2. Shape the package layout.
   - Put the main skill entrypoint at `skills/<name>/SKILL.md`.
   - Put support files under `assets/`, `examples/`, or `templates/`.
   - Keep scripts inert if present.

3. Declare every file explicitly.
   - List each packaged file in `spec.files`.
   - Do not rely on hidden globbing or implicit discovery.
   - Preserve the package tree shape exactly.

4. Add the consumer `Prayfile` entry.
   - Declare the package source.
   - Opt into the skill export.
   - Render to `.agents/skills` or the target skills directory.

5. Resolve and render.
   - Resolve the package deterministically.
   - Render the skill into the target repository.
   - Keep churn minimal and output stable.

6. Verify the result.
   - Run `pray verify` to check checksums, marker positions, and lockfile consistency.
   - Run `pray drift` when you want a broader drift report.

7. Package and publish.
   - Build the package with `pray package`.
   - Publish it with `pray publish` to a configured distribution point.

## Output

A successful run produces:

- a normalized skill directory
- a `*.prayspec` that declares the skill files and exports
- a `Prayfile` that opts the skill into a target
- a rendered copy under `.agents/skills/<name>`
- a locked, verifiable state in `Prayfile.lock`
- a publishable `.praypkg`

## Practices

- Prefer small, reviewable diffs.
- Keep `SKILL.md` as the canonical entrypoint.
- Keep provenance in `Prayfile.lock`, not in rendered files.
- Do not execute package code.
- Prefer explicit paths over magic.
- Treat manual edits to managed output as drift unless the workflow says otherwise.

## Good defaults

- Name the skill after the behavior it provides.
- Keep supporting files close to `SKILL.md`.
- Include examples only when they help use or review the skill.
- Use `render mode: :managed` for repository-shared output.
- Use `conflict: :fail` unless there is a strong reason to do otherwise.
