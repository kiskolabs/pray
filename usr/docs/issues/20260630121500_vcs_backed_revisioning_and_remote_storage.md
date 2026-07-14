# VCS-backed revisioning and optional remote storage for prayers

## Overview

Pray should be able to use a configured version-control backend as the revision store for prayers, so changes can be tracked, reviewed, and restored with normal repository workflows.

## Desired capabilities

- Support Git first, with Mercurial and other repository backends through a thin adapter layer
- Record prayer changes as ordinary VCS revisions, commits, or tags
- Keep the existing lock, verify, render, and provenance model unchanged
- Optionally sync stored prayers to a configured remote when one is set
- Preserve deterministic rendered output and hash verification across all supported backends

## Motivation

Many teams already use version control as the source of truth for instruction files, templates, and related context.

If Pray can store and revision prayers through a repository backend, then users get branching, diffs, rollback, history, and remote replication without needing a separate storage model for the same content.

## Non-goals

- Hidden background pushes or automatic remote writes
- A hard dependency on Git specifically
- Executing package code through the storage backend
- Changing the existing static, hash-verified package model

## Open questions

- Should this be an optional backend for `pray serve`, a separate command, or both?
- Should remote sync operate per package, per project, or per repository?
- What is the minimum backend contract that covers Git, Mercurial, and future systems cleanly?
