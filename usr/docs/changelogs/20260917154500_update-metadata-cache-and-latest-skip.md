# Update metadata cache and latest skip

## Participants

Andrei Makarov

## Decisions

Cache HTTP package metadata and distribution settings in process for one CLI invocation. Skip the second --latest resolve when Prayfile constraints and upstream pins do not change. Still refresh path-fork files when that path reports a write. Probe torrent sidecars only when v1/distribution.json lists torrent.

Search summaries stay names-only until an RFC 0060 change. Compact index write and git fetch-byte benches stay out of this pass.

## Effects

--latest no longer GETs the same package JSON twice. A no-op --latest is one resolve plus a path-fork refresh check. Artifact download without torrent listed does not GET .praytorrent.json.

## Next

Search names-only default or index summaries remains an RFC 0060 amendment. Git catalog refresh still needs a fixture that counts fetch bytes.

## Source

usr/docs/issues/20260917152100_update-and-index-efficiency.md
crates/pray-core/src/registry_http_cache.rs
crates/pray-core/src/fetch.rs
crates/pray-cli/src/commands_update_latest.rs
