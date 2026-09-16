# Install vs update vs verify for local compose sources

## Participants

Andrei Makarov

## Decisions

Keep local compose re-embed on pray install, plan, and apply. Do not route it through pray update. Update re-resolves package versions and git source revisions. That is a different risk class from rewriting a dest span from a human-owned file already listed in Prayfile.

Record the checkable claims from the 2026-09-16 discussion and the contract split that follows from RFC 0020, RFC 0030, RFC 0031, and the current CLI.

Help rewrite for install, update, verify, plan, and render landed in crates/pray-cli/src/help_text.rs with matching Ruby and TypeScript pages.

## Effects

Outcome: partially supported.
Source: crates/pray-core/src/verify/mod.rs collect_verification_report and drift_project; rfcs/0030-render-markers-ownership.md sections 52 and 53; rfcs/0020-resolve-and-lock.md section 32.2.
Notes: verify compares dest markers to lock ideal_checksum and line positions. It also errors when a required local file is missing. It does not compare dest to the current local file when dest still matches the lock. A same-length local edit with dest and lock still on the old bytes is clean under verify. Drift reports renderer_drift when dest differs from a fresh render, which includes current local files. Position drift can name a local path as cause when unmarked preamble and marker lines moved. After locals became managed spans, dest vs lock mismatch on that span is custom_implementation and the recovery text says run pray install. If verify failed in the original report, the likely mechanisms are position drift after a line-count change, or drift being used as verify. Check: edit .agents/project.md without changing dest or lock, run pray verify then pray drift.

Outcome: supported as a documentation defect.
Source: crates/pray-cli/src/help_text.rs WORKFLOW_COMMANDS and PACKAGE_COMMANDS; cargo run --offline -q -- --help and cargo run --offline -q -- update --help (2026-09-16).
Notes: install help says resolve packages, render targets, and update Prayfile.lock. Update help says refresh package versions within constraints, then spends four sentences on --latest. Neither names local compose files. Neither names git distribution-point revision refresh. The top-level Packages list gives add, remove, unlock, vendor, and clean a description. The update line is only flags.

Outcome: unsupported as a missing command help page. Partially supported as empty top-level description.
Source: crates/pray-cli/src/help.rs maybe_print_help; crates/pray-cli/src/help_text.rs command_help_text "update"; observed stdout of pray update --help and pray help update.
Notes: both print the long page. The concise help line has no trailing description, unlike every other Packages command.

Outcome: unsupported.
Source: rfcs/0020-resolve-and-lock.md section 37; crates/pray-cli/src/commands_update.rs update_resolve_options and update_command_with_manifest_constraints.
Notes: pray update with no package name sets ignore_locked_versions and refresh_source_revisions and re-resolves the whole graph within constraints. A package argument unlocks that name only. --major requires a package name. Default update is the broader command.

Outcome: partially supported.
Source: crates/pray-cli/src/commands_materialize.rs materialize_command; crates/pray-core/src/render_patch.rs; rfcs/0031-ownership-and-generated-output.md idempotency; usr/docs/issues/20260916172100_local-compose-checksum-report.md.
Notes: install always renders, then writes dest via patch, then writes the lock if it changed. Patch keeps unmarked dest text and replaces overlapping managed spans. Before local spans existed, a local body lived unmarked, so dest and lock looked unchanged. After local spans, a local edit changes managed_span source_checksum and ideal_checksum, so the lock moves and dest is rewritten. RFC 0031 says install may skip writes when lock and inputs are unchanged. Local files are render inputs. Dest rewrite is gated on patch output, not on package version movement.

Outcome: supported by RFC 0031 and RFC 0020 install render step. Contested as operator preference that install should not rewrite much.
Source: rfcs/0031-ownership-and-generated-output.md (install: lock and inputs unchanged, no writes; local file edits change local embeds and do not require resolve); rfcs/0020-resolve-and-lock.md section 36 (install renders target files); rfcs/0030-render-markers-ownership.md section 43 (re-embed on each render run).
Notes: lock movement here is span checksums, not package versions. That is the install class, not the update class. --locked fails when those span checksums would change. --frozen also fails when dest would change.

Outcome: contested. The risk ranking is supported. The assignment of local re-embed to update is not.
Source: crates/pray-cli/src/commands_update.rs update_resolve_options (ignore_locked_versions when no package name, refresh_source_revisions true); crates/pray-core/src/resolve_git_sources.rs (refresh fetches remote HEAD instead of the pinned lock revision); rfcs/0020-resolve-and-lock.md section 37; rfcs/0114-package-upstream.md.
Notes: default pray update can bump every locked version the constraint still admits, advance git catalog revisions, and refresh path-fork trees, then rewrite dest. Using that command to re-embed .agents/project.md couples a dest span rewrite to package and source churn. That is the double purpose problem. Install keeps locked package versions and pinned git revisions (aside from the missing-catalog-package refresh fallback).

Outcome: partially supported.
Source: crates/pray-cli/src/help_text.rs update page; crates/pray-cli/src/commands_update.rs; rfcs/0020-resolve-and-lock.md section 37; rfcs/0114-package-upstream.md; crates/pray-cli/tests/package_upstream.rs; crates/pray-cli/tests/install_git_catalog.rs.
Notes: --latest is over-described in help and is the constraint-rewrite path. Default update without --latest is implemented: re-resolve within current constraints, refresh git source revisions, refresh path-fork upstream. That is already two jobs on one verb (constraint-range version movement, and distribution-point revision movement). --latest is a third job (rewrite Prayfile and spec.upstream pins). Local compose re-embed would be a fourth. Do not add the fourth.

Related unsupported help claim, observed while checking: pray render --help said render targets without updating the lockfile. crates/pray-cli/src/commands_verify.rs render_command writes dest and Prayfile.lock. Help now names that write. Ruby render still writes dest only.

Help rewrite (2026-09-16): Packages list describes update. update --help names default graph re-resolve, package-scoped unlock, git revision refresh, path-fork refresh, and --latest. It says listed local compose files are re-embedded by pray install. install --help keeps locked versions and re-embeds listed locals. verify --help is dest versus lock and points source versus dest at drift and plan.

Confirming checks: cargo test --offline -p pray-cli --test cli_ux (9 passed). bundle exec rspec spec/pray/cli_help_spec.rb from rubygems/pray-cli (7 examples, 0 failures). node --test dist/cli/help.test.js after npm run build (help suite 6 passed). cargo fmt --all -- --check (exit 0). cargo clippy --offline -p pray-cli --all-targets -- -D warnings (exit 0). make loc-check (152 warnings, 0 failures).

## Next

Do not move local re-embed onto pray update. If dest rewrite still feels too sharp for install, the conservative CI gate is already pray install --locked and --frozen, not a different rewrite command.

Reconcile RFC 0030 section 43 and RFC 0108 unmarked locals with the managed local span from 20260916172100. That remains open.

## Source

usr/docs/issues/20260916172100_local-compose-checksum-report.md
rfcs/0020-resolve-and-lock.md
rfcs/0030-render-markers-ownership.md
rfcs/0031-ownership-and-generated-output.md
rfcs/0114-package-upstream.md
crates/pray-cli/src/help_text.rs
crates/pray-cli/src/commands_update.rs
crates/pray-cli/src/commands_materialize.rs
crates/pray-cli/src/commands_verify.rs
crates/pray-core/src/verify/mod.rs
crates/pray-core/src/resolve_context.rs
