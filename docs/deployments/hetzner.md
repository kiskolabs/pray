# Hetzner deployment

Hetzner is the most flexible option if you want full control.

## Recommended shape

- run `pray serve` directly or in Docker on a VM or dedicated server
- bind to `127.0.0.1` or `0.0.0.0` behind a reverse proxy
- use Caddy, Nginx, or Traefik for TLS and auth
- store data on local SSD, mounted volume, or object storage

## `pray serve`

```sh
pray serve --root /srv/prayers --host 127.0.0.1 --port 7429
```

## Docker Compose

```yaml
services:
  pray:
    build:
      context: .
      dockerfile: docs/deployments/pray.Dockerfile
    command: ["serve", "--root", "/data/prayers", "--host", "0.0.0.0", "--port", "7429"]
    volumes:
      - pray_data:/data
    restart: unless-stopped

  caddy:
    image: caddy:2
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile:ro
      - caddy_data:/data
      - caddy_config:/config
    depends_on:
      - pray
    restart: unless-stopped

volumes:
  pray_data:
  caddy_data:
  caddy_config:
```

## Caddyfile

```caddyfile
prayers.example.com {
  encode zstd gzip
  reverse_proxy pray:7429
}
```

## Operational notes

- this is a good fit for private registries and mirrors
- it is also the easiest route if you want custom moderation or admin controls
- back up archives, metadata, and logs separately

## When Hetzner is a good fit

- you want full control
- you need durable storage you manage directly
- you want custom auth or moderation rules
- you want the simplest path to a production-like self-hosted server