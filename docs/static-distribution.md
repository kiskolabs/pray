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

This writes index, metadata, and bytes under `./distribution`. Host that tree with any static file server, object storage, or `pray serve --root ./distribution`.

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
