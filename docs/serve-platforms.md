# Deploying `pray serve`

`pray serve` is a self-hosted distribution-point server. The specification defines the server behavior, but not a single hosted platform.

Use the platform guides below for concrete deployment templates:

- [Heroku](deployments/heroku.md)
- [Fly.io](deployments/flyio.md)
- [Hetzner](deployments/hetzner.md)
- [Cloudflare](deployments/cloudflare.md)
- [Dokku](deployments/dokku.md)

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

- **Heroku**: fine for small deployments with external storage
- **Fly.io**: strong general-purpose choice
- **Hetzner**: best for full control and durability
- **Cloudflare**: best as a proxy and security layer in front of another host
- **Dokku**: best if you want Heroku-like deploys on your own server

If you want a concrete setup next, start with the platform guide that matches your hosting choice.