# Private serving of prayers on the public internet

## Participants

Andrei Makarov

## Decisions

Private serving is not one switch. Split four properties: who can read artifact bytes, who can read index metadata, who can see consumer IPs, and whether hashes still bind the bytes. RFC 0050 already verifies hashes and optional signatures. Those prove identity of content. They do not gate download.

Pray install today issues unauthenticated HTTP GET. Edge login (Cloudflare Access IdP), HTTP Basic, service-token headers, and signed query strings therefore do not work with `pray install` until the client sends those credentials. Do not document them as working install paths.

What works now without a Prayfile change: keep the origin off the public routing table and give consumers a private overlay address (Tailscale Serve, WireGuard, VPC). The public internet carries tunnel packets. `source` is an HTTPS URL that only resolves or connects inside the overlay. `pray+ssh://` is the other working public-internet private path: SSH authenticates the session.

Do not announce a private package on the public BitTorrent DHT. BEP 5 stores peer contacts under an infohash. Anyone with the infohash learns IPs. RFC 0062 already refuses `enable_dht` and forbids off-root announce unless `v1/distribution.json` lists it.

A later install-auth RFC is required before Cloudflare Access service tokens, HTTP Basic, or Bearer-on-GET become product. That RFC must keep secrets out of Prayfile.lock (RFC 0050). Headers or a secret store beat `https://user:pass@host` in the manifest.

## Effects

Research pass on 2026-09-16. No product code changed. Complements usr/docs/issues/20260916225000_git-free-distribution.md, which covered public static hosts.

### What private means here

A hostname on the public DNS is still public serving if anyone can GET `v1/index.json`. Binding `pray serve` to 0.0.0.0 without auth is public serving. Signing packages is not private serving: signatures refuse fakes, they do not hide bytes.

Four independent controls:

1. Reachability: can an untrusted host open a TCP or UDP path to the origin
2. Request auth: does the origin require a secret or identity on each GET
3. Swarm membership: who may learn peer IPs for an infohash
4. Payload concealment: are the stored bytes ciphertext

Pray today covers 1 when the operator uses an overlay or SSH, and 4 never. 2 is Bearer on publish only. 3 is unused because swarm announce is unimplemented.

### Overlay networks (works with current install)

The origin listens on localhost or a tailnet address. Consumers run a VPN client. `source` in Prayfile is `https://prayers.ts.net` or a 100.x address. Install GETs as today. No HTTP auth in Pray.

Tailscale Serve (docs fetched 2026-09-16): `tailscale serve 3000` proxies `http://127.0.0.1:3000` to `https://<device>.<tailnet>.ts.net`, reachable only inside the tailnet. Funnel is the public opposite. Serve and Funnel cannot share the same port. ACLs apply. Identity headers are added for tailnet traffic; Funnel does not add them. Bind `pray serve` to 127.0.0.1 so those headers cannot be spoofed from the LAN.

WireGuard and a cloud VPC are the same pattern without a vendor DNS name: encrypt the path, do not publish a listener on 0.0.0.0:443.

Cloudflare Tunnel to localhost is a public hostname unless Access or an equivalent gate sits in front. README already describes the tunnel. Access in front of it is a browser gate unless the CLI sends service-token headers. So tunnel-plus-Access is not an install path today. Tunnel without Access is public HTTPS to whatever `pray serve` allows on GET.

### SSH over the public internet (works, experimental)

`source "team", "pray+ssh://pray@prayers.example"` (RFC 0104). The host is public. The session is not. Trust policy can pin host keys and publisher keys. This is private serving of the registry protocol, not of a static CDN tree.

### Edge identity that works for browsers, not for `pray install`

Cloudflare Access Allow policies send unauthenticated clients to a login page (typically HTTP 302). `pray install` does not complete that login. It GETs and fails.

Cloudflare R2 tutorial Protect an R2 Bucket with Cloudflare Access (fetched 2026-09-16): create the Access application before connecting a custom domain, otherwise connecting the domain makes the bucket public by default. Browser test expects an Access login. Note in that tutorial: anonymous users should use pre-signed URLs, not Access membership.

Cloudflare Access service tokens (docs fetched 2026-09-16): Client ID and Client Secret as `CF-Access-Client-Id` and `CF-Access-Client-Secret`. Policy action must be Service Auth, not Allow. Optional `read_service_tokens_from_header` can map that pair into one `Authorization` header. Pray HTTP clients send neither. A Cloudflare Access application in front of Pages, R2, or `pray serve` therefore blocks install until a client change.

HTTP Basic at nginx or Caddy is the same gap: the proxy wants `Authorization: Basic`. Pray does not send it. Putting `https://user:pass@host` in Prayfile would also fight RFC 0050 (no secrets in the lockfile) and would leak in error strings that interpolate the URL.

### Signed URLs and IAM (not an install source today)

CloudFront signed URLs and cookies (AWS private-content docs, fetched 2026-09-16) attach expiry and a signature to each object URL. Prayfile `source` is one origin prefix, reused for index, package JSON, and artifacts, and committed. A time-limited per-object URL does not map. S3 pre-signed URLs have the same shape problem. R2 docs recommend pre-signed URLs for anonymous users; that is a download link, not a registry origin.

CloudFront origin access control can keep the S3 bucket private and let only CloudFront read it. Viewers still need either a public distribution or signed URLs. A public CloudFront URL in front of a private bucket is still public serving of the bytes.

IAM user keys for GetObject would need the HTTP client to sign AWS requests. Pray does not.

### Capability URLs (weak)

An unguessable subdomain or path prefix hides the tree from casual crawlers. The URL still lands in Prayfile, clone history, and CI logs. Treat as obscurity. Do not call it private serving.

### Torrent, private trackers, DHT

Pray torrent fetch is HTTP Range against the origin. Privacy of that path equals privacy of the HTTP origin. Listing `bootstrap_trackers` does not talk to a tracker.

If a later swarm wire ships:

Public DHT (BEP 5) is a public peer directory. Announcing a private package there publishes consumer IPs under the infohash. That is the opposite of private serving. Keep `enable_dht` failing for private roots.

Private torrents (BEP 27, fetched 2026-09-16): `private=1` in the info dict. Clients MUST announce only to the private tracker and MUST only initiate connections to peers that tracker returns. DHT, PEX, and LSD would subvert the tracker ACL. The tracker is the membership list. BEP 27 also states the limit: once an intruder has a peer IP and port, that peer will trade pieces. Private torrent is tracker ACL plus disabled gossip, not encryption.

Magnet links (BEP 9) without `tr` SHOULD use the DHT. Magnet plus `private=1` is a poor pair: metadata is not private until after DHT lookup. A private swarm should distribute the descriptor out of band (HTTPS on the overlay, or SSH), not as a public magnet.

A private tracker (OpenTracker or Chihaya on a tailnet, or passworded HTTP announce) only helps after Pray implements announce. Until then, do not run a public tracker for private packages.

### Tor, I2P, and other overlays

RFC 0104 lists Tor and I2P as future transports, not RFC 0062 protocol names. An onion service is reachability privacy: the origin has no public A record. Install would need a SOCKS transport. Unimplemented. Same class as Tailscale: hide the listener, keep GET unauthenticated inside the overlay.

### Signatures versus download gates

`pray trust set-require-signed-packages` refuses unsigned remotes. A stolen CDN copy still installs if the signature matches. Private serving is who may fetch. Trust policy is whose bytes you accept after fetch.

### Practical operator order

1. Team on one overlay: Tailscale Serve or WireGuard to `pray serve --host 127.0.0.1` or a static nginx of `publish --root`. Prayfile `source` is the tailnet HTTPS URL. Works today.
2. Need public DNS and a process: `pray+ssh://` with host-key pin. Works today, experimental.
3. Need a public HTTPS CDN and closed membership: wait for install-auth (Access service tokens or Basic or Bearer-on-GET) plus secrets outside the lockfile. Does not work today. Cloudflare Access on R2/Pages is a browser gate only.
4. Need consumers who never share a VPN: that is a product RFC, not an operator trick. Public DHT will not provide it.

## Next

If private CDN install is wanted, write an RFC for request authentication on registry GET: header names, secret storage (`PRAY_HOME` or OS helper, not Prayfile.lock), and parity across Rust, Ruby, and TypeScript. Cloudflare Access service tokens and HTTP Basic are the two smallest external contracts to map.

Do not implement public DHT for private packages.

Keep overlay recipes in operator docs. State the Access/CLI gap next to the existing tunnel section so operators do not expect IdP login to unlock `pray install`.

## Source

RFC 0050 avoid secrets in lockfile; optional signatures. RFC 0051 Bearer on enroll and publish-adjacent auth, not install GET. RFC 0062 private roots must not announce off-root. RFC 0104 pray+ssh. `crates/pray-core/src/registry_http.rs` GET with no Authorization. Ruby `Net::HTTP::Get` without basic_auth. TypeScript `fetch(url)` without headers.

Tailscale Serve, https://tailscale.com/docs/features/tailscale-serve , fetched 2026-09-16. Serve is tailnet-only. Funnel is public. Same port cannot be both. Outcome: supported.

Cloudflare Access service tokens, https://developers.cloudflare.com/cloudflare-one/access-controls/service-credentials/service-tokens/ , fetched 2026-09-16. Headers CF-Access-Client-Id and CF-Access-Client-Secret. Service Auth policy required. Outcome: supported.

Cloudflare R2 plus Access tutorial, https://developers.cloudflare.com/r2/tutorials/cloudflare-access/ , fetched 2026-09-16. Create Access before connecting the custom domain. Browser login. Pre-signed URLs for anonymous users. Outcome: supported.

Amazon CloudFront private content overview, https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/private-content-overview.html , fetched 2026-09-16. Signed URLs or cookies for viewer restriction. OAC for bucket restriction. Outcome: supported.

BitTorrent BEP 27 Private Torrents, https://bittorrent.org/beps/bep_0027.html , fetched 2026-09-16. private=1. Tracker-only announce. DHT/PEX/LSD subvert ACL. Intruder with IP still trades. Outcome: supported.

BitTorrent BEP 5, previously fetched for usr/docs/issues/20260916225000_git-free-distribution.md. Public DHT stores peer contacts. Outcome: supported.

usr/docs/issues/20260916180900_distribution-torrent-sidecar-opt-in.md: today's torrent sidecar is not a public DHT announce.
