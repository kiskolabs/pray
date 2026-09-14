# CLI invocation context leak on main CI

## Participants

Andrei Makarov

## Decisions

Restore the previous invocation context when Ruby CLI.run and TypeScript runCli return, including on errors. Keep the same in-process contract on both CLIs. Do not change how a process-per-command install discovers the project from the working directory or --path.

## Effects

CI run 34838034114 failed the ruby job on main at 0ae6aad. The example provisioned destination safety keeps the previous lock when destination materialization fails raised ENOENT while listing /tmp/pray-git-distribution-*/consumer/packages/shell. git_distribution_spec and provisioned_dest_spec share polyrun shard 2. CLI.run left the git consumer as the thread invocation context after the workspace was deleted. materialize_project then resolved the second Prayfile against that leftover root.

bundle exec rspec spec/pray/cli_run_context_spec.rb failed before the restore with Invocation.context still pointing at the deleted first project. After restore, that example passed. bundle exec rspec spec/pray/cli_run_context_spec.rb spec/pray/git_distribution_spec.rb spec/pray/provisioned_dest_spec.rb spec/pray/install_git_global_cache_spec.rb spec/pray/invocation_spec.rb finished 20 examples, 0 failures. bundle exec rubocop lib/pray/cli.rb spec/pray/cli_run_context_spec.rb --cache false reported no offenses. bundle exec polyrun parallel-rspec --workers 5 --merge-failures finished 5 workers, exit 0. npm run lint in npmjs/pray-cli reported no fixes. node --test dist/cli-run-context.test.js passed. npm test in npmjs/pray-cli finished 167 tests, 0 failed. make loc-check finished 141 warnings, 0 failures. CLI.run is 125 lines. runCli is 247 lines.

## Next

Push patch/cli-invocation-context-restore and open a pull request when asked. Confirm the ruby job on GitHub.

## Source

https://github.com/kiskolabs/pray/actions/runs/34838034114
rubygems/pray-cli/lib/pray/cli.rb
npmjs/pray-cli/src/cli/main.ts
usr/docs/issues/20260914140500_prepare-1-14-0-release.md
