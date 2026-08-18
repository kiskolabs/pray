# RFC 0060: Distribution

- Feature Name: distribution
- Type: Standards Track
- Status: Stable
- Describes: 1.8.1
- Created: 2026-08-17
- Author: Andrei Makarov
- Relates: RFC 0050, RFC 0101, RFC 0104

## Summary

Packages move through static registry trees, `pray serve`, publish and yank, substring search, and transport adapters. Every hop MUST verify artifact and tree hashes. Federation wire details belong in RFC 0104.

## Motivation

Teams need public, private, local, and optional peer distribution. Verify semantics stay the same across transports.

## Guide-level explanation

`pray publish --root` writes a static tree a CDN can host. `pray serve --root` adds HTTP (and optional stdio) for local or self-hosted points. `pray sync` pulls from explicit peers. `pray search` queries an index by substring match (changelog 1.8.0).

Static hosting without `pray serve` MUST remain valid. CI can publish to a directory and install from that directory.

Yank MUST be metadata-only.

Clients SHOULD see one install flow regardless of transport. Clients install from one URL. Mirrors change availability.

## Reference-level explanation

This section is the product contract for this concern. Where it disagrees with Implementation notes, Implementation notes record what the reference CLI does today. A follow-on RFC records the gap.

### 29. Static registry protocol

The registry may be a static file tree.

Recommended layout:

```
/v1/index.json
/v1/packages/sample/base.json
/v1/packages/sample/webapp.json
/v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg
```

index.json:

```json
{
  "spec": "prayfile-distribution-1",
  "packages": [
    "sample/base",
    "sample/webapp"
  ]
}
```

Package metadata:

```json
{
  "name": "sample/base",
  "versions": [
    {
      "version": "1.4.3",
      "artifact": "v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg",
      "artifact_hash": "sha256:...",
      "tree_hash": "sha256:...",
      "yanked": false,
      "targets": ["tool_a", "tool_b", "generic"],
      "exports": ["working-agreements", "testing-basics"],
      "derived": {
        "languages": ["markdown"],
        "encodings": ["utf-8"],
        "origins": ["git+ssh://git@example.com/base.git"],
        "summary": "Shared operational guidance for agent use",
        "categories": ["policy", "workflow"],
        "topics": ["testing", "review", "migrations"],
        "file_count": 12,
        "character_count": 18420,
        "token_count": 4120,
        "possible_effects": ["reduce drift", "standardize review output"],
        "possible_side_effects": ["narrower phrasing", "more explicit workflow bias"],
        "embeddings": [
          {
            "model": "local-or-cloud-derived",
            "scope": "package"
          }
        ]
      },
      "confessions": {
        "published_by": "example-maintainer",
        "collected_by": ["sample/base", "prayers.kisko.dev"],
        "received": 18
      }
    }
  ]
}
```

No server API is required for v1. Static hosting must be enough.
### 28. Sources

Supported source kinds: registry, static index, git, local path, tarball, OCI artifact, pray SSH

Examples:

```
source "default", "https://agents.example.com"
source "sample", "git+ssh://git@example.com/agent-context/index.git"
source "team", "pray+ssh://pray@prayers.internal"
source "local", path: "../agent-packages"
```

Direct package sources:

```
agent "sample/base", git: "git+ssh://git@example.com/base.git", tag: "v1.4.3"
agent "local/base", path: "../base"
agent "public/base", tarball: "https://example.com/base-1.4.3.praypkg"
agent "public/base", oci: "registry.example.com/agents/base:1.4.3"
```

---

### 30. Registry metadata fields

A registry package version should expose:

name, version, summary, description, artifact location, artifact hash, tree hash, yanked flag, license, homepage, source code URI, changelog URI, targets, exports, dependencies, published_at optional, signature optional, signer optional, signer_fingerprint optional, signer_public_key optional, render_digest optional, annotation_provenance optional

Remote registry installs must fail closed when `artifact_hash` or `tree_hash` is missing. Signature verification rules are defined in Section 59.

To reduce churn and privacy leakage, project lockfiles should not copy unnecessary registry metadata.

---
#### 29.3 Derived metadata and confessions

Distribution points may compute and publish derived metadata for each package version. Derived metadata is an annotation layer, not package identity. It does not change the artifact hash, tree hash, or version identity.

Derived metadata may be computed locally, through cloud inference, or by combining both. Implementations may use language detection, encoding detection, summary generation, topic extraction, embedding generation, and similar analysis tools.

Derived metadata may include:

- detected languages
- detected encodings
- source origins and provenance notes
- summary
- categories
- topics mentioned
- file count
- character count
- token count
- possible effects
- possible side effects
- embeddings

A package may consist only of minimal editable text files intended for alteration. The distribution point may enrich that package with derived metadata without requiring the package itself to carry those annotations.

Confessions are signed usage feedback records. A confession may be produced by a publisher, a distribution point, or a client that has received a package. Confessions may be collected, mirrored, and aggregated by publishers and trusted servers.

Federated servers may share known peer and server lists, along with confessions they are authorized to relay. Publishers may use confessions to collect usage feedback across direct publication and server-to-server synchronization.

Confessions do not alter package identity. They are feedback data attached to a package version, tree hash, or artifact hash.
#### 29.4 Zero-trust verification and engine-agnostic annotations

Pray assumes zero trust. Any client, server, publisher, or federation peer may provide incorrect, partial, stale, or malicious data. All metadata, summaries, scores, confessions, and derived annotations are claims unless independently verified or explicitly accepted under local policy.

Package authenticity and injection safety must be verified separately:

- package bytes are verified with artifact hashes and signatures
- package trees are verified with normalized tree hashes
- final injected bytes are verified with exact render digests or equivalent deterministic byte checks
- render plans are verified with canonical metadata about selected exports, exclusions, ordering, normalization, and target policy

Derived metadata may be used for verification, but only as evidence. It can help prove what should be injected, what was excluded, and which inputs were used. It does not become truth simply because it is published by a server.

Any participant may generate annotations using any method, including manual review, hardcoded logic, deterministic heuristics, local inference, cloud inference, or generative models. Implementations must record annotation provenance when they rely on such output, including the producer, method, policy or model version, and the input hash or equivalent binding used to generate the claim.

If conflicting claims are received, local policy decides which claims to trust for discovery or display. Verification of the final injected bytes remains mandatory.

Clients remain unaware of federation. A client queries a single distribution point, which may serve from local mirror, proxy to a peer, or return metadata with origin URLs.
#### 29.6 Client git source trust policy

A conforming client implementation may enforce optional trust policy for remote `git` sources before resolving packages from a cloned distribution repository.

Policy file location (reference CLI): `~/.pray/trust.toml`. Override with `PRAY_HOME` or `PRAY_USER_HOME` per implementation.

```toml
[default]
allow = true
require_signed_commit = false
require_signed_packages = false
allowed_signing_keys = []

[[rules]]
match_prefix = "https://github.com/example/"
require_signed_commit = true
require_signed_packages = true
allowed_signing_keys = ["SHA256:ABCDEF..."]
```

Longest `match_prefix` wins; otherwise `[default]` applies.

When policy exists, the client may:

- deny sources with `allow = false`
- require `git verify-commit HEAD` when `require_signed_commit = true`
- refuse remote registry packages without a signature when `require_signed_packages = true`
- restrict signers to `allowed_signing_keys` when that list is non-empty
- prompt for consent when HEAD has no verified-good signature and the signer is not already trusted

SSH-signed commits should use per-source `allowedSignersFile` values scoped to the client's own git subprocesses (`$PRAY_HOME/trust/allowed_signers/`), without modifying the user's global git configuration.

For `pray+ssh` sources, trust rules may also set `allowed_host_keys` (server host key fingerprints) and `allowed_publishers` (SSH user key fingerprints allowed to publish). Package metadata should record `signer_fingerprint` separately from the human-readable `signer` label; signatures use the fingerprint when present.

Reference CLI commands: `pray trust list|show|add-key|remove-key|set-signed|set-require-signed-packages|set-allow|import-repo|import-registry|check`. `pray trust import-registry` reads `v1/ssh_publishers.json` from a distribution point and records publisher fingerprints in `allowed_publishers` for the matching rule; for `pray+ssh` sources it also records the server host key in `allowed_host_keys` unless `--no-host-key` is passed. `pray trust check` compares trusted keys against a compromised-key feed (HTTP URL, local file, or git repository). Distribution operators may mint scoped publish tokens with `pray token create --root PATH --email EMAIL` for `Authorization: Bearer` on HTTP push (`PRAY_PUBLISH_TOKEN`). Clients may run `pray search <query>` over a declared index without marketplace ranking.

Global flags: `--trust` imports signer keys after interactive consent; `--global --trust` imports into `[default]`. `PRAY_TRUST_ASSUME_YES=1` auto-consents in non-interactive environments. `--rm` uses an ephemeral `PRAY_HOME` but still copies persistent trust policy into it.

---

## Implementation notes

Changelog 1.8.0 added yank, scoped publish tokens, search, and HTML-free packaging smoke. Beta E2E covers publish, consume, sync, and verify on HTTP.

Production-readiness still wants a two-machine network path with injected failure. Inference from operator notes; no fixture in-tree for that path at this writing.

## Security considerations

`serve --allow-open-push` is a trust expansion. Default SHOULD require tokens or enrolled keys. Federation trust is explicit peer config.

## Unresolved questions

Which transports are conformance Level 3 versus experimental extras. Recommendation: HTTP static, git, path, and tarball are Level 3; SSH, torrent, P2P, and federation stay experimental until RFC 0104 is Stable.
