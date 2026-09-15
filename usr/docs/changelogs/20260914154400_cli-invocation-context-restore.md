# Restore invocation context after in-process CLI run

## Participants

Andrei Makarov

## Decisions

CLI.run and runCli restore the invocation context that was active when the command started. Nested commands keep the outer project. A later install in another directory uses that directory.

## Effects

Ruby CLI.run and TypeScript runCli no longer leave the previous project selected after return. The CI ruby failure on main at Record v1.14.0 tag was this leftover context plus polyrun shard pairing of git_distribution_spec and provisioned_dest_spec.

## Next

Later pass 20260915103600: this work ships as 1.15.0. See usr/docs/issues/20260915103600_prepare-1-15-0-release.md.

## Source

usr/docs/issues/20260914154400_cli-invocation-context-restore.md
usr/docs/issues/20260915103600_prepare-1-15-0-release.md
https://github.com/kiskolabs/pray/actions/runs/34838034114
