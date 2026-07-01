# Deploying `pray serve`

`pray serve` is a self-hosted distribution-point server. The specification defines the server behavior, but not a single hosted platform.

Use the deployment guides below for concrete templates:

- managed hosting
- general-purpose hosted app platform
- dedicated server
- edge proxy or tunnel in front of another host
- self-managed app server

## Common requirements

A `pray serve` deployment should be treated as a stateful web service:

- bind to the host and port provided by the platform
- store package archives and metadata on durable storage
- keep audit logs in durable storage
- put TLS and authentication in front of publish/admin paths when exposed publicly
- do not rely on an ephemeral filesystem for package storage

The specification allows `pray serve` to expose package lookup, downloads, checksums, signatures, publisher identity, publishing, yanking or deprecation metadata, signed usage feedback, and optional human-readable package pages.

## Protections

The specification describes a zero-trust supply-chain model.

Recommended protections for a hardened distribution point:

- account authentication for publishing
- two-factor authentication
- passkey support
- explicit passkey check before publishing
- package signatures on publish
- SSH signing key support
- immutable package archives after publish
- yanking or deprecation instead of silent replacement
- append-only audit logs for publish events
- checksum verification before and after upload

Consumers should also verify package hashes, signatures, and render digests or equivalent deterministic byte checks.

## Admin UI

The specification does not define a formal admin UI.

It does allow optional human-readable package pages and confession review or moderation workflows, so an admin console is an implementation choice, not a standardized part of `pray serve`.

A practical admin UI would usually include:

- publish package
- yank or deprecate a version
- review confessions
- inspect signatures and checksums
- view audit logs
- manage trusted publishers or peers
- inspect replication status if federation is enabled

## Suggested deployment choice

- managed hosting: fine for small deployments with external storage
- general-purpose hosted app platform: strong default choice
- dedicated server: best for full control and durability
- edge proxy or tunnel: best as a layer in front of another host
- self-managed app server: best if you want app-server-style deploys on your own server

If you want a concrete setup next, start with the guide that matches your hosting choice.