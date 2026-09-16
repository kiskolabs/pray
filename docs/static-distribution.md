# Static distribution discovery

Pray v1 distribution points are a static file tree. No server API is required for install.

## Layout

Given a distribution root (directory or HTTP origin):

```text
v1/index.json
v1/packages/{namespace}/{name}.json
v1/artifacts/{namespace}/{name}/{version}/{name-slug}-{version}.praypkg
```

Discovery from a root URL or path:

| Role | Location |
|------|----------|
| Package list | `{root}/v1/index.json` |
| Package metadata (versions, hashes, yanked, signatures) | `{root}/v1/packages/{ns}/{name}.json` |
| Immutable artifact bytes | `{root}/v1/artifacts/.../*.praypkg` |
| Optional write / sync | `pray serve`, `pray publish --server`, or `pray+ssh` |

Clients verify `artifact_hash` and `tree_hash` on remote installs. Signatures are optional in v1 but verified when present.

## Publish without a server (CI-friendly)

```bash
pray package
pray publish --root ./distribution --signing-key "$PRAY_SIGNING_KEY"
```

This writes index, metadata, and bytes under `./distribution`. Host that tree with any static file server, object storage, or `pray serve --root ./distribution`. Git is not required. A consumer Prayfile points `source` at the HTTP origin:

```
source "public", "https://prayers.example.com"
pray "sample/base", "~> 1.4"
```

Install GETs `{origin}/v1/index.json`, package metadata, and the `.praypkg`. It verifies `artifact_hash` and `tree_hash`. Publish into a static host is a file upload or object sync, not `pray publish --server`.

### Cloudflare Pages

Deploy the published directory as a Pages project.

Add a top-level `404.html` in that directory. Without it, Pages treats the upload as a single-page app and can serve `/` HTML for `/v1/index.json`.

Pages currently answers Range requests with `200` and the full file. `pray install` accepts that: RFC 0062 says a 200 full body is the whole artifact, then hashes still run. Spec-compliant `206` is not required for install.

### Cloudflare R2

Upload the same `v1/` keys into an R2 bucket. Enable a custom domain on a zone in the same account for production. The `*.r2.dev` development URL is rate-limited and not a production origin.

Install never lists the bucket root. Known paths are enough.

### Amazon S3

Upload the same keys. Prefer the REST object endpoint, or CloudFront HTTPS in front of the bucket. S3 website endpoints (`s3-website.<region>.amazonaws.com`) do not support SSL and return HTML errors.

S3 GetObject honors a single `Range` and returns `206 Partial Content`. That matches piece fetch when `v1/distribution.json` lists `protocols: ["torrent"]`.

### Direct archive

A single package can skip the index with `tarball: "https://example.com/sample-base-1.4.3.praypkg"` or a local `.praypkg` path. That is one artifact, not a versioned catalog.

### `pray serve` and tunnels

Use `pray serve` when you need HTTP publish, confess, or sync. A Cloudflare Tunnel in front of localhost `pray serve` is documented in the README. That path still runs a process. Pages and object storage replace only GET of the tree.

### Private origins on the public internet

`pray install` GETs the v1 tree with no `Authorization` header. Package signatures prove who published; they do not hide bytes.

Working today:

- Overlay: bind `pray serve` or a static file server to localhost, publish it on Tailscale Serve or WireGuard. Prayfile `source` is the tailnet HTTPS URL. The public internet carries tunnel packets, not a public v1 listener.
- `pray+ssh://` with host-key pinning. The SSH host is public; the session is not.

Not working today for install:

- Cloudflare Access identity-provider login (browser 302)
- Access service-token headers (`CF-Access-Client-Id` / `CF-Access-Client-Secret`)
- HTTP Basic at a reverse proxy
- CloudFront or R2 pre-signed URLs (expiry and per-object query strings do not match a committed source origin)
- Public BitTorrent DHT (peer IPs under an infohash). Keep DHT off for private roots.

Create a Cloudflare Access application before attaching an R2 custom domain, or the bucket becomes public. That Access gate still only helps browsers until install-auth exists.

### Torrent, trackers, DHT, magnet

`protocols: ["torrent"]` writes `.praytorrent.json` beside the artifact. Install then Range-GETs pieces from the same HTTP origin, or accepts a 200 full body.

That is not BitTorrent. Tracker URLs in `bootstrap_trackers` are copied into the descriptor and are not announced. `enable_dht: true` fails. Magnet URIs are not a source kind. Swarm announce, DHT, and magnet remain future work (RFC 0062, RFC 0104).

## Yank

```bash
pray yank sample/base 1.4.3 --root ./distribution
pray yank sample/base 1.4.3 --root ./distribution --undo
```

Yank flips the `yanked` flag in package metadata only. Artifact bytes stay immutable. New resolves skip yanked versions. Locked installs may continue with a warning; `pray install --strict` refuses them.

## Local override for offline roots

In `PRAY_CONFIG` / `$PRAY_HOME/config.toml`:

```toml
[local.source]
default = "../distribution"
```

Use with a Prayfile `source "default", "https://example.invalid"` (or any registry URL) so resolve reads the local tree without HTTP.

## Scoped publish tokens (HTTP `--server`)

Mint a pasteable token against a distribution root auth database (user must already exist):

```bash
pray token create --root ./distribution --email publisher@example.com --scope publish
export PRAY_PUBLISH_TOKEN='…'
pray publish --server https://prayers.example --signing-key "$PRAY_SIGNING_KEY"
```

The client sends `Authorization: Bearer …` on artifact upload and sync push. Loopback bind and `--allow-open-push` remain available for local development. Revoke with `pray token revoke --root ./distribution TOKEN`.

## Require signed packages

```bash
pray trust set-require-signed-packages --match-prefix https://prayers.example --enabled true
```

When enabled for a source prefix, remote resolve fails if package metadata has no signature.

## Search

```bash
pray search base --root ./distribution
pray search web --url https://prayers.example
pray search sample --source default
```

Substring match on package names from `v1/index.json`. Optional summaries come from package metadata. No ranking.
