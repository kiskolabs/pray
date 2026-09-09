# TLS and HTTP/2 lock freshness

## Participants

Andrei Makarov

## Decisions

Apply the compatible Cargo lock bump and toml-rb patch from usr/docs/issues/20260904113000_dependency-audit.md. Add a local make audit that runs the three CI scanners. Leave zstd and ring in place. Do not add a second HTTP, TLS, archive, or TOML stack. Skip CHANGELOG.md: lock freshness and contributor audit targets do not change a user-facing contract.

## Effects

Cargo.lock now pins rustls 0.23.43, h2 0.4.19, hyper 1.11.1, aws-lc-rs 1.18.1, aws-lc-sys 0.45.0, and toml 1.1.5. Several Windows crates retargeted windows-sys 0.52.0 to 0.61.2 as a compatible side effect of that update.

rubygems/pray-cli/Gemfile.lock now pins toml-rb 4.2.1.

Makefile gained audit, audit-rust, audit-ruby, and audit-npm. audit-npm is npm audit --omit=dev. A later local npm audit of the full graph, including development dependencies, reported zero vulnerabilities after several minutes.

Validation:

- cargo update -p rustls -p h2 -p hyper -p aws-lc-rs -p toml finished with a compatible lock of six packages.
- cargo test --workspace finished successfully.
- make audit-rust finished with advisories, bans, licenses, and sources ok. Duplicate-version and unused-license warnings match the audit pass.
- In rubygems/pray-cli, bundle update toml-rb locked 4.2.1.
- make audit-ruby reported no vulnerabilities. ruby-advisory-db commit 0a02e5f.
- In rubygems/pray-cli, bundle exec polyrun parallel-rspec --workers 5 --merge-failures finished with all five shards exit 0.
- In npmjs/pray-cli, gtimeout 25 npm audit --omit=dev printed found 0 vulnerabilities and EXIT 0.
- In npmjs/pray-cli, npm audit --omit=dev then npm audit printed found 0 vulnerabilities twice (exit 0 after about 286 seconds). npm view reported semver 7.8.5 and smol-toml 1.8.0.
- make audit as one target was not run. audit-npm can sit quiet for minutes before returning.

## Next

Watch ring and zstd. Keep TypeScript on 5.9 until madge accepts a newer peer.

## Source

usr/docs/issues/20260904113000_dependency-audit.md
Cargo.lock
Makefile
rubygems/pray-cli/Gemfile.lock
