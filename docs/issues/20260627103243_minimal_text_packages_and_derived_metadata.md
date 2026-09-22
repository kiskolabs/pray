# Minimal Text Packages, Derived Metadata, and Confessions

## Overview

The Pray specification should allow packages to consist only of minimal editable text files, while the distribution point computes richer package metadata from those files.

The package itself remains static and parseable. The server-derived layer can include language detection, encoding detection, summaries, categories, topics, counts, possible effects, possible side effects, embeddings, and confession records.

## Motivation

This keeps packages lightweight and human-editable while moving expensive or variable analysis into the distribution layer.

It also separates package identity from derived annotations and usage feedback.

## Scope

- Clarify that text-only packages are valid
- Define derived metadata as annotation, not identity
- Describe confessions as signed feedback records
- Allow publishers and servers to collect and relay confessions
