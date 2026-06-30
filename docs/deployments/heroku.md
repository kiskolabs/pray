# Heroku deployment

Heroku works well for a small registry-like service if package storage lives outside the dyno filesystem.

## Recommended shape

- run `pray serve` as the web process
- bind to `0.0.0.0:$PORT`
- store artifacts in external object storage or a database-backed store
- keep publishing behind authentication
- use Heroku’s router for TLS

## Procfile

```procfile
web: pray serve --root /app/prayers --host 0.0.0.0 --port $PORT
```

## Operational notes

- Heroku dyno filesystems are ephemeral
- use durable storage for archives, metadata, and audit logs
- if you need publish or admin access, place it behind authentication or a private network boundary
- keep read-only package browsing public if that fits your use case

## Suggested add-ons or services

- object storage for package archives
- managed database for metadata and audit logs
- private access control for publish endpoints

## When Heroku is a good fit

- small private registry
- simple public mirror
- minimal ops overhead
- external storage already available