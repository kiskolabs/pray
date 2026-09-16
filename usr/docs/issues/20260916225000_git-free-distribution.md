# Git-free prayer distribution: static hosts, object storage, trackers, DHT, magnet

## Participants

Andrei Makarov

## Decisions

Git is optional. RFC 0060 already says a distribution point is a static file tree. `pray publish --root DIR` writes that tree. A consumer points `source` at the HTTP origin or a local path. No git provider is required for publish or install.

Do not put live Cloudflare, S3, or public BitTorrent DHT in default CI. Those paths need accounts, money, NAT, and wall-clock discovery. They fail for reasons unrelated to Prayfile.

Do add in-repo end-to-end fixtures that replay the real HTTP contracts: a static tree served without `pray serve`, an origin that returns 200 for Range (Cloudflare Pages today), and an origin that returns 206 with Content-Range (S3 GetObject and RFC 0062). That is the proof that git-free install works. Swarm tracker, DHT, and magnet fixtures wait until those wires exist.

A running `pray serve` behind a Cloudflare Tunnel remains valid. It is a different product: a process with publish, confess, and sync. Pages and object storage replace only the read path.

## Effects

Research pass on 2026-09-16. No product code changed.

### What already works without git

`pray publish --root` writes `v1/index.json`, package JSON, and `.praypkg` bytes. `docs/static-distribution.md` already tells operators to host that directory with any static file server or object store.

A consumer Prayfile uses a registry URL, not a git URL:

```
source "public", "https://prayers.example.com"
pray "sample/base", "~> 1.4"
```

Install then GETs `{origin}/v1/index.json`, `{origin}/v1/packages/{ns}/{name}.json`, and the artifact path in that metadata. Remote installs fail closed without `artifact_hash` and `tree_hash`.

Other git-free sources that already exist:

- local path (`path:`)
- a `.praypkg` on disk or over HTTP (`tarball:`)
- `pray+ssh://` (RFC 0104 experimental; still a server, not git)
- `pray serve` on a machine you control, optionally published through a tunnel

`oci:` on a package declaration is still unsupported. RFC 0060 lists it. The 1.16 tarball work left it out.

### Cloudflare Pages

Pages can host the published tree if every URL maps to a file. Two origin facts from Cloudflare Pages serving docs, fetched 2026-09-16:

- Pages currently returns 200 for HTTP range requests, not spec-compliant 206. RFC 0062 already requires clients to accept a 200 full body as the whole artifact when the server ignores Range, then still verify piece and artifact hashes. Pages is therefore usable for install today, including when `protocols` lists `torrent`.
- If the project has no top-level `404.html`, Pages treats the upload as a single-page app and matches unknown paths to `/`. That would replace `v1/index.json` and package JSON with HTML. Operators must ship a top-level `404.html` so missing paths stay 404.

Recipe:

1. `pray repo init` or `pray publish --root ./distribution`
2. Add `distribution/404.html` so Pages does not SPA-fallback
3. Deploy the `distribution` directory as a Pages project
4. Point `source` at `https://<project>.pages.dev` or the custom domain

Publish into Pages is a file upload or CI wrangler deploy, not `pray publish --server`. Yank is rewrite of package JSON in the same tree, then another deploy.

### Cloudflare R2

R2 public buckets serve object keys over HTTP. Official R2 docs (fetched 2026-09-16): buckets are private until the operator enables a custom domain or an `r2.dev` development URL. Production wants a custom domain on a zone in the same account. `r2.dev` is rate-limited and meant for non-production.

Upload the same `v1/...` keys. Consumer `source` is the custom domain. JSON is not in Cloudflare's default cached file types; operators who want edge cache for `index.json` must set a cache rule. That is an operator choice, not a Prayfile requirement.

R2 does not list bucket contents at the domain root. Pray install never lists the root. It GETs known paths. That mismatch is harmless.

### Amazon S3

S3 GetObject documents a Range header and a sample 206 Partial Content response (AWS API reference, fetched 2026-09-16). S3 does not support multiple ranges on one GET. Pray asks for one range per piece. That matches.

S3 website endpoints (`s3-website.<region>.amazonaws.com`) support only anonymous GET and HEAD, return HTML errors, and do not support SSL. Prefer the REST object endpoint or CloudFront HTTPS in front of the bucket. Map keys to the same `v1/...` paths. Block public listing if the bucket policy allows GET of known keys only; install still works because clients never list.

### `pray serve` versus static hosting

`pray serve` is required for HTTP publish, confess, search HTML, and federation sync. Static Pages/R2/S3 replace GET of the tree. They do not replace `pray publish --server`.

README already documents a Cloudflare Zero Trust Tunnel in front of localhost `pray serve`. That path still needs a running process. Use it when you want write APIs. Use Pages or object storage when you only need install.

### Torrent protocol today

RFC 0062 `protocols: ["torrent"]` writes `{artifact}.praytorrent.json` with spec `pray-torrent-v1`, 16KiB piece SHA-256, artifact hash, and length. Descriptor `sources` stay relative artifact paths on the same origin. Install GETs the descriptor, then Range-GETs pieces from those HTTP URLs, or accepts a 200 full body.

`bootstrap_trackers` copies into the descriptor `trackers` array. Fetch does not read that array. `crates/pray-core/src/registry_torrent.rs` downloads from `sources` or `artifact_url` only. Listing a tracker URL does not announce, scrape, or talk BEP-3/BEP-15.

`enable_dht: true` fails parse with "DHT announce is not implemented". Confirmed by `crates/pray-core/tests/distribution.rs` `dht_flag_fails`.

`pray-transport` `P2PTransport` is `FederationTransport` renamed `p2p`. RFC 0104 says that in one sentence. It is not BitTorrent, not a DHT, and not magnet.

### Tracker setup (future wire, not product yet)

A BitTorrent tracker is a peer-introduction server. It does not store `.praypkg` bytes. Clients announce an infohash and receive other peers' IP and port.

OpenTracker binds TCP and UDP 6969 by default and serves `/announce` and `/scrape` (erdgeist opentracker project page). Chihaya is a Go HTTP and UDP tracker with YAML config (chihaya/chihaya README).

When swarm announce ships, an operator who wants a tracker would:

1. Run OpenTracker or Chihaya on a reachable host
2. Put `http://host:6969/announce` or `udp://host:6969/announce` in `v1/distribution.json` `bootstrap_trackers`
3. Keep `protocols: ["torrent"]`
4. Keep DHT off unless a later RFC turns it on

Until the CLI speaks announce/scrape, those URLs are inert metadata. Do not tell operators a tracker is live.

Private roots must not list public trackers unless the same file opts in. RFC 0062 already states that. A later question is whether non-loopback tracker URLs must be refused on a private root.

### Trackerless: DHT and magnet (future wire)

BitTorrent BEP 5 (accepted, last-modified 2020-01-21): a DHT stores peer contacts for trackerless torrents. Each node is a UDP Kademlia participant. `get_peers` looks up an infohash. `announce_peer` stores the announcer. A trackerless `.torrent` has `nodes` instead of `announce`. Newly installed clients need bootstrap contacts in the torrent or routing table. BEP 5 tells implementers not to automatically add `router.bittorrent.com`.

BitTorrent BEP 9 (accepted, magnet section): `xt` is the only mandatory magnet parameter. Format `magnet:?xt=urn:btih:<info-hash>` (40 hex characters, or 32-character base32). Optional `dn`, `tr`, `x.pe`. If no tracker is specified, the client SHOULD use the DHT (BEP 5) to acquire peers. Metadata can be fetched from peers over `ut_metadata` so a `.torrent` file is not required first. BEP 9 metadata blocks are 16KiB. Pray's piece size is also 16KiB; that is coincidence of constant, not proof the wires match. Pray pieces hash `.praypkg` bytes with SHA-256. BitTorrent v1 infohash is SHA-1 of the bencoded info dict.

A magnet therefore does not replace `v1/index.json`. It names one swarm. Package name, version, yank, and signature still live on a distribution tree or an equivalent index. A later design can put a magnet or infohash on package metadata as an extra fetch key. Identity stays RFC 0050 `artifact_hash` (SHA-256). A btih SHA-1 is a fetch locator, not package identity.

WebTorrent, BitTorrent v2 (`urn:btmh:`), and browser webrtc swarms are named in RFC 0062 future possibilities under the existing `torrent` protocol name. None of them ship.

### P2P that is interesting and still honest

The interesting P2P product is: a consumer who already has `artifact_hash` can find bytes from other consumers when the origin is down, then still verify SHA-256 and signatures.

That is not what the code does. Today's "torrent" is origin HTTP with piece checksums. Today's "p2p" transport is federation HTTP under another name.

A useful sequence, if this work is funded:

1. Keep static HTTP as the Level 3 install path
2. Prove Range-honoring and Range-ignoring origins in CI
3. Then a loopback swarm: two processes, one seed, one leech, hash verify, no public DHT
4. Then optional tracker announce from `bootstrap_trackers`
5. Then optional DHT with operator-supplied bootstrap nodes, default off
6. Magnet or infohash on package metadata last, because discovery without an index is a different product

IPFS, iroh, metalink, and blossom stay later allowlist names on the same `protocols` field. RFC 0062 already lists them.

### End-to-end testing

Already in tree:

- `crates/pray-cli/tests/install_beta_e2e.rs`: publish, consume, sync on loopback `pray serve`
- `crates/pray-cli/tests/install_distribution_point.rs`: HTTP serve plus git distribution repo
- `crates/pray-cli/tests/install_publish.rs`: torrent descriptor write and skip
- `crates/pray-core/tests/registry.rs`: torrent Range fixture
- RFC 0060 implementation notes: production still wants a two-machine network path with injected failure; no fixture in-tree

Missing, and worth adding:

- publish `--root`, then install from a static HTTP server that is not `pray serve` (python http.server, miniserve, or a tiny test listener)
- same tree behind a listener that ignores Range and returns 200 (Pages contract)
- same tree behind a listener that returns 206 and Content-Range (S3/R2/`pray serve` contract)
- Pages SPA trap: missing `404.html` must fail install if the host maps unknown paths to `/`; that fixture belongs only if the test host can emulate Pages

Not worth adding to default CI:

- wrangler Pages deploy
- live R2 or S3 bucket
- public DHT lookup
- real OpenTracker on the internet

Optional later, out of default CI: a manual or nightly job with a throwaway R2 bucket and a documented `source` URL, behind secrets the repo already refuses to store.

## Next

Add a static-origin install test that does not start `pray serve`. Cover 200-for-Range and 206-for-Range. Keep it loopback.

Private origins on the public internet: overlay and pray+ssh work with current unauthenticated GET. Cloudflare Access, HTTP Basic, and public DHT do not. See usr/docs/issues/20260916230100_private-serving-public-internet.md.

If Cloudflare Pages is a documented operator path, add `404.html` to the published distribution example so a Pages deploy cannot SPA-fallback. That is a docs and fixture change, not a CLI flag.

Open question: whether `bootstrap_trackers` should fail parse until fetch uses them, so operators cannot believe a tracker URL is live. Current RFC copies them into the descriptor and leaves fetch experimental.

## Source

RFC 0060 static registry protocol. RFC 0062 distribution protocols. RFC 0104 P2PTransport is FederationTransport named p2p. `docs/static-distribution.md`. `docs/serve-platforms.md`. README Cloudflare tunnel section.

Cloudflare Pages serving docs, https://developers.cloudflare.com/pages/configuration/serving-pages/ , fetched 2026-09-16. Quotes: "Pages currently returns `200` responses for HTTP range requests; however, the team is working on adding spec-compliant `206` partial responses." And: "If your project does not include a top-level `404.html` file, Pages assumes that you are deploying a single-page application." Outcome: supported.

Cloudflare workers-sdk issue 3861, Support for 206 Partial Content, still open on 2026-03-31 comment, https://github.com/cloudflare/workers-sdk/issues/3861 . Outcome: supported as corroboration that Pages 206 is unfinished.

Cloudflare R2 public buckets, https://developers.cloudflare.com/r2/buckets/public-buckets/ , fetched 2026-09-16. Private by default. Custom domain or r2.dev. r2.dev rate-limited, non-production. No root listing. Outcome: supported.

Amazon S3 GetObject, https://docs.aws.amazon.com/AmazonS3/latest/API/API_GetObject.html , fetched 2026-09-16. Range header. Sample response HTTP/1.1 206 Partial Content. No multiple ranges per GET. Outcome: supported.

Amazon S3 website endpoints, https://docs.aws.amazon.com/AmazonS3/latest/userguide/WebsiteEndpoints.html , fetched 2026-09-16. Website endpoint GET/HEAD only, public content, no SSL. Outcome: supported.

BitTorrent BEP 5 DHT Protocol, https://www.bittorrent.org/beps/bep_0005.html , fetched 2026-09-16. Trackerless peer contacts over UDP Kademlia. Outcome: supported.

BitTorrent BEP 9, https://bittorrent.org/beps/bep_0009.html , fetched 2026-09-16. Magnet `xt` mandatory. No tracker means SHOULD use DHT. Outcome: supported.

OpenTracker project page, http://erdgeist.org/arts/software/opentracker and gitweb sample config. Default 0.0.0.0:6969 TCP and UDP, /announce and /scrape. Outcome: supported for how a tracker process is run. Not evidence that Pray talks to it.

Chihaya README, https://github.com/chihaya/chihaya/ . HTTP and UDP tracker. Outcome: supported as software that exists. Not a Pray integration.

`crates/pray-core/src/registry_torrent.rs` fetch uses sources, not trackers. `crates/pray-core/src/distribution.rs` `enable_dht` fails. `crates/pray-transport/src/p2p.rs` wraps federation. Outcome: supported.

usr/docs/issues/20260626183000_torrent_seeding_and_collective_dht_distribution.md asked for future DHT docs. README already has the future-tense P2P sentence. SPEC.md is gone; RFCs 0060, 0062, and 0104 replaced it.

usr/docs/issues/20260916180900_distribution-torrent-sidecar-opt-in.md recorded that today's sidecar is not a public DHT announce.
