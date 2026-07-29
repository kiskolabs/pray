# Ruby pray-cli polyrun QC and login/confess/sync/trust

## Participants

- Andrei Makarov

## Decisions

- Adopt action_reporter-style polyrun QC in rubygems/pray-cli: polyrun.yml, coverage gate, Makefile lint/test, RBS, CI polyrun run.
- Coverage tracks lib except CLI dispatch, plan, and ssh_agent; minimum line percent starts at 80 and should rise toward 85 as library specs grow.
- Implement HTTP-first login, confess, and sync parity with Rust; keep pray_ssh resolve, serve --stdio, and pray_ssh confess/sync deferred.
- Expand trust.toml schema with require_signed_commit and allowed_signing_keys; expose full trust CLI subcommands.

## Effects

- rubygems/pray-cli tests run under polyrun; root make ruby-test and CI ruby job use that path.
- login writes .pray/session.json; confess posts signed ConfessionSubmission; sync pulls HTTP federation peers into a distribution root.
- serve answers federation discovery, sync index/package, and confession POST.
- trust CLI mutates and checks client trust policy.

## Next

- Raise polyrun coverage minimum to 85 once more library paths are exercised.
- Port pray_ssh transport for confess, sync, and trust import-registry --host-key when needed.
- Add serve --stdio when Ruby needs stdio peer parity.

## Source

- rubygems/pray-cli/
- crates/pray-cli/src/auth_client.rs
- crates/pray-cli/src/confess.rs
- crates/pray-cli/src/sync_command.rs
- crates/pray-cli/src/trust_command.rs
- crates/pray-core/src/client_trust/
