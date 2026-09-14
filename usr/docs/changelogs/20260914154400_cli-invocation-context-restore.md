# Restore invocation context after in-process CLI run

## Participants

Andrei Makarov

## Decisions

CLI.run and runCli restore the invocation context that was active when the command started. Nested commands keep the outer project. A later install in another directory uses that directory.

## Effects

Ruby CLI.run and TypeScript runCli no longer leave the previous project selected after return. The CI ruby failure on main at Record v1.14.0 tag was this leftover context plus polyrun shard pairing of git_distribution_spec and provisioned_dest_spec.

## Next

Open a pull request from patch/cli-invocation-context-restore when asked.

## Source

usr/docs/issues/20260914154400_cli-invocation-context-restore.md
https://github.com/kiskolabs/pray/actions/runs/34838034114
