# npm CLI login, upgrade, update flags, and trust import

## Participants

- Andrei Makarov

## Decisions

- Port login, upgrade, update/plan/outdated flags, and trust import-repo/import-registry into the TypeScript npm CLI to close advertised gaps versus the Rust reference CLI.
- npm upgrade installs via `npm install -g pray-cli@latest` rather than `cargo install`.
- trust import-registry supports local roots and http(s); pray+ssh host-key import stays unsupported until SSH transport exists in TypeScript.

## Effects

- Added passkey and ssh-agent login with `.pray/session.json` persistence.
- Added `pray upgrade` and help/suggest entries.
- Wired `update --major|--latest|--dry-run|--json`, `plan --remote`, and `outdated --remote`.
- Implemented `trust import-repo` from cached git HEAD signing metadata and `trust import-registry` from `v1/ssh_publishers.json`.
- Added focused unit tests; `npm test` passed (75 tests).

## Next

- Optional: full materialization preview report parity for `plan`.
- Optional: pray+ssh registry import and serve --stdio.

## Source

- Spec: SPEC.md auth and trust sections; Rust reference in crates/pray-cli and crates/pray-core
- Code: npmjs/pray-cli/src/auth, npmjs/pray-cli/src/cli/commands/update*.ts, npmjs/pray-cli/src/trust/import-*.ts
