# Canonical Verification Records for Injection and Confession Claims

## Overview

Pray should define a canonical verification record shape for package verification, render-plan verification, final injected-byte verification, and confession claims.

This record should bind each claim to stable hashes, provenance, policy, and producer identity so clients and servers can compare what was claimed with what was actually injected or received.

## Motivation

Zero-trust operation requires a stable way to record what was verified, by whom, using which method, and against which inputs.

A canonical record keeps the protocol engine-agnostic while still allowing metadata to support verification.

## Scope

- Define a canonical verification record format
- Bind claims to package, render, and confession identities
- Record producer, method, policy, and input hash provenance
- Keep verification data separate from package identity