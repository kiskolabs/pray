# Drift reports renderer_drift after install with a trailing local compose fragment

## Participants

vesan

Andrei Makarov

## Decisions

Keep dest versus a fresh unpatched render as the renderer_drift comparison. Fix the dest write path so a patched dest with no extra unmarked notes matches that fresh render.

When dest already has overlapping managed spans and compose adds a later span, copy the unmarked text that sits between those spans in the fresh render, including the blank line after each span. When dest already has both spans with no text between them, restore that blank from fresh on the next install.

Do not treat this as a verify bug. Dest versus lock stayed honest.

## Effects

Issue 28 reproduction, checked with a path package fixture matching the compose order (package, then local), not the live kiskolabs/prayers.git clone.

pray 1.19.0: supported. Workspace version in Cargo.toml is 1.19.0.

Exit code 6 for drift findings: supported. docs/cli-exit-codes.md maps 6 to verify failed, also when pray drift finds drift.

After a clean install with a trailing local fragment, dest differs from a fresh render: supported. patch_rendered_content appended the new managed span and dropped the blank line that ContentBuilder leaves between spans. layout_rendered_targets wrote that patched dest and relocated lock line numbers to it. verify --strict compared dest to that lock and stayed clean. drift compared dest to unpatched render_project output and failed with renderer_drift.

pray render then byte-identical dest: supported as the same patch path. A second render patches the already-patched dest and writes the same bytes.

pray plan unchanged and drift --semantic clean: supported. plan uses layout_rendered_targets. --semantic only prints package version arrows.

Whenever a compose block includes a local-path fragment: partially supported. A first install onto a missing dest returns the fresh render wholesale, so from-scratch local compose is clean. The false positive needs dest that already has overlapping package spans, then a new span after them. The issue reproduction does that by installing the package-only block first. The same gap would apply to a new package added after existing spans, not only to a local file.

Lockfile records the local fragment as a managed span with source_checksum of the on-disk file: supported. That is the 1.17.0 local span record. It is not the cause. The cause is trailing-span patch omitting fresh unmarked text between spans.

CI cannot gate on pray drift for a compose block with project-local overrides: partially supported. It fails when the local fragment is added after an existing dest span. A block that listed the local first, or a dest written from scratch, did not hit this gap. After this fix, install writes dest that matches fresh, and a later install restores a dest that already lacked the blank line.

Confirming checks: cargo test --offline -p pray-core --lib patch_tests (5 passed). cargo test --offline -p pray-cli --test install_local (5 passed, including drift_is_clean_after_adding_a_trailing_local_compose_fragment). cargo test --offline -p pray-cli --test install_drift (5 passed). cargo fmt --all -- --check (exit 0). cargo clippy --offline -p pray-core --all-targets -- -D warnings (exit 0). cargo clippy --offline -p pray-cli --all-targets -- -D warnings (exit 0). bundle exec rspec spec/pray/render_patch_spec.rb spec/pray/destination_render_spec.rb from rubygems/pray-cli (17 examples, 0 failures). bundle exec rubocop lib/pray/render_patch.rb spec/pray/render_patch_spec.rb (no offenses). npx biome check src/render/patch.ts src/render/patch.test.ts in npmjs/pray-cli (clean). node --test dist/render/patch.test.js after npm run build (patchRenderedContent 2 passed). make loc-check (165 warnings, 0 failures). Full npm test was not used as evidence: sandbox git hook writes failed in unrelated suites.

## Next

Open a pull request for patch/drift-local-compose-fragment citing issue 28.

A dest already written by 1.19.0 without the blank line heals on the next pray install after this change. Deleting the dest is not required.

## Source

https://github.com/kiskolabs/pray/issues/28

usr/docs/issues/20260916180700_verify-drift-render-check.md

usr/docs/issues/20260916172100_local-compose-checksum-report.md

crates/pray-core/src/render_patch.rs

crates/pray-core/src/verify/mod.rs
