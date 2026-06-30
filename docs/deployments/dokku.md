# Dokku deployment

Dokku is a good choice if you want Heroku-like deploys on your own server.

## Recommended shape

- deploy `pray serve` as a web process
- bind to `0.0.0.0:$PORT`
- mount persistent storage for packages and metadata
- use Dokku’s proxy layer or an external reverse proxy for TLS

## Procfile

```procfile
web: pray serve --root /data/prayers --host 0.0.0.0 --port $PORT
```

## Example Dokku setup

```sh
dokku apps:create pray
dokku storage:mount pray /var/lib/dokku/data/storage/pray:/data/prayers
dokku config:set pray PRAY_ROOT=/data/prayers
```

If you use a reverse proxy or custom domain, add the usual Dokku domain and TLS setup for your host.

## Operational notes

- the Dokku host filesystem is persistent, but app containers should still treat writable layers as temporary
- keep archives, metadata, and audit logs on mounted storage or an external service
- publishing should stay behind authentication

## When Dokku is a good fit

- you want simple self-hosted deploys
- you already run your own Linux server
- you want a Heroku-like workflow without a managed PaaS