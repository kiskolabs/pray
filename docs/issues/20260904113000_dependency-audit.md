# Dependency audit 2026-09-04

## Participants

Andrei Makarov

## Decisions

Full-depth audit of the Pray workspace at version 1.11.0: Rust crates pray-core, pray-cli, pray-transport, and pray-bench; Ruby gem rubygems/pray-cli; npm package npmjs/pray-cli. Smallest follow-up is a compatible Cargo lock bump of rustls, h2, hyper, aws-lc-rs, and toml, plus a one-patch toml-rb bump. Do not replace zstd or ring. Do not add a second HTTP, TLS, archive, or TOML stack.

Hot-path Rust direct crates are reqwest (rustls, blocking or json, no http3), ed25519-dalek, sha2, getrandom, rusqlite (optional auth), tokio, tar, zstd, serde, serde_json, toml, semver, base64, and libc on unix. Transport also uses async-trait, anyhow, and thiserror. Ruby runtime is toml-rb and base64. npm runtime is semver and smol-toml.

Later pass on 2026-09-04: take the HTTP-path Cargo lock bump, take toml-rb 4.2.1, and add a local make audit that mirrors the three CI scanners. CHANGELOG.md is unchanged because this is lock freshness and contributor tooling, not a user-facing contract. A delayed npm audit of the full graph, including development dependencies, later finished with zero vulnerabilities.

## Effects

cargo deny check advisories licenses bans sources reported advisories, bans, licenses, and sources ok. Warnings were duplicate crate versions (base64 0.22 and 0.23, getrandom 0.3 and 0.4, hashbrown 0.16 and 0.17, r-efi 5 and 6, syn 2 and 3, windows-sys 0.52 and 0.61) and unused license allow-list entries (0BSD, BSD-2-Clause, Unicode-DFS-2016).

In rubygems/pray-cli, bundle exec bundler-audit check --update reported no vulnerabilities. ruby-advisory-db commit 0a02e5f, last updated 2026-08-31. bundle exec rake libyears reported 3.2 libyears behind. Runtime lag that matters was toml-rb 4.2.0 versus 4.2.1 (0.3 years). Remaining lag is development or major: rubocop-performance 1.26.1 versus 1.27.0, diff-lcs 1.6.2 versus 2.0.0, rbs 3.10.4 versus 4.2.0, and interpreter 3.4.7 versus 4.0.6. Locked base64 0.3.0 matches RubyGems latest under constraint ~> 0.2.

cargo-libyear --sort libyear --top 20 reported 80.61 libyears across 304 packages, about 0.265 years average. Band used is the high-churn Cargo table in the dependency-audit libyears note: totals near 1000 can still be healthy; 80 is treated as low. Coverage gate remains cargo llvm-cov --fail-under-lines 65. cargo-outdated was not installed; freshness used crates.io comparison plus the libyear tool.

npm audit --omit=dev reported found 0 vulnerabilities on the audit pass, after the lock bump (gtimeout 25, EXIT 0), and again in a later production scan (exit 0 after about 173 seconds). npm audit including development dependencies finished later in the same session with found 0 vulnerabilities (exit 0 after about 286 seconds for omit-dev then full then npm view). registry.npmjs.org on 2026-09-04: semver 7.8.5 and smol-toml 1.8.0 match the lock. typescript latest is 7.0.2 while the package stays on ~5.9.3 because madge 8.0.0 does not accept that peer. @biomejs/biome latest 2.5.12 versus lock 2.5.11 is development only.

crates.io versus the pre-bump Cargo.lock on 2026-09-04: reqwest, ed25519-dalek, sha2, rusqlite, tokio, tar, zstd, serde, semver, getrandom 0.4.3, and chacha20 match latest. Lag on the HTTP path was rustls 0.23.41 versus 0.23.43, h2 0.4.16 versus 0.4.19, hyper 1.10.1 versus 1.11.1, aws-lc-rs 1.17.1 versus 1.18.1 (aws-lc-sys 0.42.0 versus 0.45.0), toml 1.1.4 versus 1.1.5. cargo update -p rustls -p h2 -p hyper -p aws-lc-rs -p toml applied a compatible lock of those six packages. Several Windows crates retargeted windows-sys 0.52.0 to 0.61.2 as a compatible side effect. cargo deny still warns on both windows-sys lines.

h2 0.4.16 was the patched floor for RUSTSEC-2026-0258 / GHSA-q83h-524g-xf6h (unbounded empty DATA frames). h2 0.4.17 further ignores EOS in the DATA budget and caps the HPACK encoder table at 4kb. rustls 0.23.43 notes a reachable panic in debug or overflow-checks when decrypting specific ticket lengths with Rfc5077Ticketer or the aws-lc-rs Ticketer, pre-auth, and QUIC client panics. Pray enables reqwest without http3, so QUIC notes are weaker. aws-lc-rs remains the rustls provider, so the ticket panic is still adjacent to the client path after the bump.

Prior lock defects recorded under usr/docs/dependencies remain resolved in this lock: chacha20 0.10.2, h2 at least 0.4.16, json at least 2.21.2.

GitHub OSINT on 2026-09-04 (open_issues_count includes pull requests): seanmonstar/reqwest push 2026-09-01, 470 open, 11806 stars; rustls/rustls push 2026-09-03, 83 open, 7594 stars; RustCrypto/hashes push 2026-09-01; rusqlite/rusqlite push 2026-08-30, 172 open, 4379 stars; tokio-rs/tokio push 2026-09-03; composefs/tar-rs push 2026-09-01 (crate repository moved from alexcrichton/tar-rs); toml-rs/toml push 2026-09-03; hyperium/hyper push 2026-09-01; hyperium/h2 push 2026-09-03; aws/aws-lc-rs push 2026-09-01; npm/node-semver push 2026-09-03; squirrelchat/smol-toml push 2026-08-23. dalek-cryptography/ed25519-dalek is archived; crates.io repository is dalek-cryptography/curve25519-dalek/tree/main/ed25519-dalek, push 2026-08-29, 1191 stars. emancipu/toml-rb (RubyGems homepage) push 2026-07-31, 11 open, 118 stars. gyscos/zstd-rs last crates.io publish 2025-02-20, last GitHub push 2026-06-24, 91 open, 653 stars. briansmith/ring last crates.io publish 2025-03-11, last GitHub push 2026-07-23, 49 open, 4106 stars. ring is still a dependency of quinn-proto 0.11.16 and rustls-webpki 0.103.13.

CI already gates cargo-deny-action (advisories, licenses, bans, sources), Ruby bundler-audit, and npm audit --omit=dev plus npm audit. Dependabot weekly covers cargo, npm, bundler, and github-actions. npm Dependabot ignores TypeScript major and @types/node major. fuzz/ is a separate Cargo tree with libfuzzer-sys.

Follow-up validation on branch patch/tls-http2-lock-freshness:

- cargo update -p rustls -p h2 -p hyper -p aws-lc-rs -p toml finished with a compatible lock of six packages.
- In rubygems/pray-cli, bundle update toml-rb locked toml-rb 4.2.1.
- cargo test --workspace finished successfully.
- make audit-rust finished with advisories, bans, licenses, and sources ok, same duplicate-version and unused-license warnings as the audit pass.
- make audit-ruby reported no vulnerabilities (ruby-advisory-db 0a02e5f).
- In rubygems/pray-cli, bundle exec polyrun parallel-rspec --workers 5 --merge-failures finished with all five shards exit 0.
- In npmjs/pray-cli, gtimeout 25 npm audit --omit=dev printed found 0 vulnerabilities and EXIT 0.
- In npmjs/pray-cli, npm audit --omit=dev later printed found 0 vulnerabilities (exit 0).
- In npmjs/pray-cli, npm audit --omit=dev then npm audit then npm view semver version and npm view smol-toml version printed found 0 vulnerabilities twice, then 7.8.5 and 1.8.0 (exit 0 after about 286 seconds).
- make audit as a single target was not run. audit-npm can sit quiet for minutes before returning.

Makefile now has audit, audit-rust, audit-ruby, and audit-npm. audit-npm is production-only (npm audit --omit=dev). CI still runs the full graph audit as well.

## Next

Watch ring and zstd for a crates.io publish or an advisory. Watch rustls-webpki and quinn for a ring drop. Keep TypeScript on 5.9 until madge accepts a newer peer.

Duplicate base64 0.22 versus 0.23 remains after the HTTP-path bump. Wait for reqwest or hyper-util to move, or re-check after the next reqwest line.

## Source

Workspace Cargo.toml 1.11.0, Cargo.lock, rubygems/pray-cli/Gemfile.lock, npmjs/pray-cli/package-lock.json, Makefile, .github/workflows/ci.yml.

usr/docs/changelogs/20260904114500_tls-http2-lock-freshness.md

Commands: cargo deny check advisories licenses bans sources; bundle exec bundler-audit check --update; bundle exec rake libyears; cargo-libyear --sort libyear --top 20; cargo update -p rustls -p h2 -p hyper -p aws-lc-rs -p toml; cargo test --workspace; bundle update toml-rb; bundle exec polyrun parallel-rspec --workers 5 --merge-failures; make audit-rust; make audit-ruby; npm audit --omit=dev; npm audit; npm view semver version; npm view smol-toml version; gtimeout 25 npm audit --omit=dev.

Registries queried 2026-09-04: crates.io API v1 for the hot-path and TLS crates; rubygems.org API for toml-rb and base64; registry.npmjs.org for semver, smol-toml, typescript, @biomejs/biome, madge.

GitHub repos queried via api.github.com: seanmonstar/reqwest, rustls/rustls, dalek-cryptography/curve25519-dalek, composefs/tar-rs, gyscos/zstd-rs, briansmith/ring, emancipu/toml-rb, plus the earlier pass on rusqlite, tokio, hyper, h2, aws-lc-rs, npm/node-semver, squirrelchat/smol-toml.

Prior notes: usr/docs/dependencies/20260901170644_chacha20-yanked-in-lockfile.md, usr/docs/dependencies/20260901143628_h2-empty-data-frame-advisory.md, usr/docs/dependencies/20260829222000_json-cve-2026-71847.md, usr/docs/dependencies/20260829222001_madge-typescript-peer.md.
