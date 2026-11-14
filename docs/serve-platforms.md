# Deploying `pray serve`

`pray serve` is a self-hosted distribution-point server. The specification defines the server behavior, but not a single hosted platform.

Choose a deployment shape from the [options below](#suggested-deployment-choice), or use the [SSH-only guide](deployments/ssh.md) when no HTTP listener is needed.

## Common requirements

A `pray serve` deployment should be treated as a stateful web service:

- bind to the host and port provided by the platform
- store package archives and metadata on durable storage
- keep audit logs in durable storage
- put TLS and authentication in front of publish/admin paths when exposed publicly
- do not rely on an ephemeral filesystem for package storage

## Protections

Apply the [publishing and signing policy](../README.md#publishing-and-signatures) and [confession submission requirements](../README.md#confession-feedback) to exposed write paths. [RFC 0050](../rfcs/0050-security-and-trust.md) defines the security baseline; [RFC 0060](../rfcs/0060-distribution.md#294-zero-trust-verification-and-engine-agnostic-annotations) defines package and rendered-byte verification.

## Admin UI

The specification allows optional human-readable package pages and confession review or moderation workflows. Implementers choose whether to provide an admin console and how it works.

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
