# Dependency upgrade QA for npm and Ruby on trunk/2026w45

## Decisions

QA the uncommitted npm and Ruby dependency upgrades already present on the parent trunk/2026w45 working tree, together with the Cargo.lock and toml floor bump already QA'd for Rust.

Do not force-downgrade madge to clear the transitive minimatch advisory. Keep typescript at ~5.9.3 for the madge peerOptional constraint. Keep @types/node on major 22 per dependabot.yml. Accept rubocop 1.91.0 lag versus 1.92.2 because standard 1.57.0 pins rubocop ~> 1.91.0. Keep rbs at ~> 4.0 (resolved 4.2.0).

Leave dependency file edits in the worktree for the parent to consolidate. Do not push.

## Effects

Worktree created from trunk/2026w45 at 3a00d807ad70ab8fffc71e38d04f0c61456a46da. Shared WORKTREE_ID=dependency-qa-2db8dbfd. WORKTREE_START_REF=trunk/2026w45. Setup skipped after checking REPO_ROOT and WORKTREE_PATH for .cursor/worktrees.json (neither present). Parent uncommitted dependency files were copied into the worktree before QA.

npmjs workspace (cd npmjs && npm ci): PASS (197 packages added).

npm tests:

- cd npmjs/pray-core && npm test: PASS (263 tests, 0 fail)
- cd npmjs/pray-cli && npm test: PASS (70 tests, 0 fail)
- cd npmjs/pray-core && npm run test:coverage: PASS (263 tests, 0 fail; all files line 77.82%, branch 74.61%, funcs 85.03%; threshold lines 60)

npm typecheck and lint:

- npm run typecheck in pray-core and pray-cli: PASS
- npm run lint in both: PASS (exit 0). Parent later aligned biome.json $schema to 2.6.5. No circular deps from madge

npm audit:

- cd npmjs/pray-cli && npm audit --omit=dev: PASS (0 vulnerabilities)
- cd npmjs && npm audit --omit=dev: PASS (0 vulnerabilities)
- cd npmjs && npm audit (full, includes dev): FAIL exit 1 with 7 moderate findings, all through madge -> dependency-tree -> precinct -> detective-typescript/vue2 -> @typescript-eslint/typescript-estree -> minimatch (GHSA-g3hv-75wg-m249). npm audit fix --force would install madge@7.0.0 (breaking). Left as known accepted risk for a lint-only cycle checker

make libyears-npm: PASS. libyear --dev --quiet in pray-cli and pray-core reported empty dependency lag tables (total blank). npm outdated still lists typescript 5.9.3 wanted/current versus latest 7.0.2, and @types/node 22.20.5 versus latest 26.6.6; both intentional pins

Ruby (rubygems/pray-cli):

- bundle install: PASS
- make test without UTF-8 locale: FAIL once on Pray::RegistrySearch encoding compare (em dash UTF-8 vs US-ASCII) under sandbox default locale
- LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8 make test: PASS (rubocop 213 files no offenses; rbs validate ok; polyrun 5 workers exit 0). Dry-run example count 389 examples, 0 failures. No coverage fragments under coverage/ for polyrun merge
- bundle exec bundler-audit check --update: PASS (No vulnerabilities found; advisory DB commit 516ea4757d5eec778c49af8371c23309c4b70f42, last updated 2026-11-04)
- bundle outdated leftovers: diff-lcs 1.6.2 vs 2.0.0; prism 1.9.0 vs 2.1.0; rubocop 1.91.0 vs 1.92.2; rubocop-performance 1.26.1 vs 1.27.0. rubocop lag expected from standard 1.57.0 pin. Major bumps for diff-lcs and prism not taken in this upgrade
- bundle exec rake libyears: system 2.5 libyears behind (rubocop-performance 0.8, prism 0.8, diff-lcs 0.7, rubocop 0.1, ruby 0.1)
- Locked versions of note: rbs 4.2.0, standard 1.57.0, rubocop 1.91.0, rubocop-thread_safety 0.8.0, polyrun 2.2.8, json 3.1.0

make size-check (after npm build and cargo build --release --locked -p pray-cli with CARGO_TARGET_DIR set to the worktree target/ because the agent sandbox redirects CARGO_TARGET_DIR): PASS

- rust_binary bytes=8976432 budget=9500000 status=ok
- ruby_gem bytes=118784 budget=131072 status=ok
- npm_packed bytes=133340 budget=150000 status=ok
- npm_unpacked bytes=574403 budget=650000 status=ok
- npm_entries count=339 budget=360 status=ok

make bench-compat: PARTIAL

- Rust compat_bench: PASS (fixture sha256 matches; peak_rss_bytes=4096000)
- Ruby compat_bench.rb: PASS (same fixture sha256; peak_rss_bytes=34848768)
- npm scripts/compat-bench.mjs via make: FAIL (ERR_MODULE_NOT_FOUND for npmjs/pray-cli/dist/index.js). pray-cli build emits only cli/ and parse.*; the bench still imports ../dist/index.js and ../dist/hashing.js. Pre-existing on origin/main as well. Workaround import from ../../pray-core/dist produced matching fixture sha256 and ran: parse p50_us=8.98 resolve p50_us=129.17 render p50_us=5.38 lock p50_us=10.00 peak_rss_bytes=86638592

Rust re-verify with present Cargo changes: cargo test -p pray-core --lib PASS (134 passed, 0 failed). toml floor 1.1 -> 1.2 confirmed.

Upgrade-resource compare versus origin/main (46a242772cb8da3e438566bac060e7a37c6b1761) on the same host, isolated baseline worktree at agentfile-main-baseline under the same WORKTREE_ID set. Probe: du -sk of installed trees; /usr/bin/time -l for node bin/pray.js --help and bundle exec pray --help; n=3 with alternate BASE/HEAD order; medians:

- npmjs/node_modules bytes: base 148426752 (142M), head 149200896 (142M), delta +774144
- separate BUNDLE_PATH gem trees bytes: base 50847744 (48M), head 52330496 (50M), delta +1482752
- node --help RSS median bytes: base 72269824, head 72351744, delta +81920
- node --help user CPU median seconds: base 0.08, head 0.08, delta 0
- bundle exec pray --help RSS median bytes: base 43991040, head 42844160, delta -1146880
- bundle exec pray --help user CPU median seconds: base 0.16, head 0.16, delta 0

No cheaper/smaller/faster claim for npm install size (head larger by about 0.77 MiB). Ruby gem tree larger by about 1.4 MiB. Boot RSS for Ruby help was slightly lower on head; treat as observed sample set, not a product claim. Shipped artifact sizes covered by make size-check budgets above.

Resolved npm production pins observed: semver 7.8.6, smol-toml 1.9.1, @biomejs/biome 2.6.5.

## Next

Parent consolidates the eight dependency files via /apply-worktree or by keeping the already-copied parent copies; this worktree also holds this issue note.

Optional follow-ups (out of scope for this upgrade unless parent wants them in the same change):

- Point npmjs/pray-cli/scripts/compat-bench.mjs at pray-core dist (or restore a pray-cli re-export) so make bench-compat passes without a workaround
- Run biome migrate / bump biome.json $schema to 2.6.5 to clear the lint info
- Set LANG/LC_ALL=UTF-8 in Ruby CI or polyrun workers so registry search string compares stay encoding-stable
- Revisit madge when upstream clears the minimatch chain without a major downgrade

## Source

Worktree path under ~/.cursor/worktrees/dependency-qa-2db8dbfd/agentfile-1e2e07fe1cc8. Branch tip trunk/2026w45. Base for resource compare: origin/main 46a2427. Related Rust QA note: docs/issues/20261106093128_rust-cargo-upgrade-qa.md. dependabot.yml comments document the typescript and @types/node pins. Advisory: https://github.com/advisories/GHSA-g3hv-75wg-m249
