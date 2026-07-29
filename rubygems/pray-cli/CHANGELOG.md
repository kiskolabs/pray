# CHANGELOG

## Unreleased

- Add `rake build` / `rake release` for gem packaging and RubyGems push.
- Rework gem summary, description, and README opener to match Prayfile positioning.

## 1.5.0 (2026-07-27)

- Add `((pray:symbol))` templating with project-wide `pray do` / `template do` symbol maps.
- Substitute symbols in rendered fragments, local embeds, and UTF-8 provisioned files.
- Accept Gemfile-like Ruby surface sugar: `{…}` blocks, top-level `;`, and optional call parentheses.
- Accept `pray` / `use` / `include` / `package` as package declaration aliases of `agent`.
- Parse and apply `compose`, `tree`, and `file:` destinations with role-based export selection and scoped render/provision.
- Exercise the shared `testdata/shared` Prayfile corpus alongside Rust and TypeScript.
- Rewrite Prayfile with `pray format` (`fmt`) to the recommended compose/tree/file destination DSL.

## 1.1.0 (2026-07-14)

- Add environment-aware rendering with `group` blocks and `--env` or `PRAY_ENV`.
- Add global `--path` and `--file-path` flags with `PRAY_PATH`, `PRAY_FILE_PATH`, and project `.env` support.
- Record the selected environment in `Prayfile.lock`.
- Improve CLI help with grouped commands, per-command help, and suggestions for unknown commands.
- Add `--no-input` to skip interactive prompts.
- Honor `PRAY_NO_COLOR` and `NO_COLOR` for plain terminal output.
- Refresh git distribution caches on install when a locked revision is missing locally.

## 1.0.0 (2026-07-13)

- Initial RubyGems release of `pray-cli`.
- Resolve local path packages and git distribution sources.
- Publish to local distribution roots and serve over HTTP.
- Install, update, render, verify, and drift workflows with `Prayfile.lock`.
- Git distribution integration tests for install, locked revision, and update.
