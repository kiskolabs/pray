# Verify, drift, and render --check are different questions

## Participants

Andrei Makarov

## Decisions

Do not combine pray verify, pray drift, and pray render --check. They compare different pairs. Dry-run of a write is pray plan, not pray render.

The operator analogies: verify as linter, drift as diff, render as dry-run. Keep them as teaching hints only after the comparisons are named.

Help rewrite landed: first sentence of each page names the pair compared. Dry-run points at pray plan. Rust and TypeScript render help match the dest and lock write. --check stays a dest identity gate.

## Effects

Outcome: unsupported.
Source: rfcs/0030-render-markers-ownership.md sections 52 and 53; rfcs/0020-resolve-and-lock.md section 32.2; rfcs/0034-unused-specified-surfaces.md; crates/pray-cli/src/commands_verify.rs; crates/pray-cli/src/lockfile_ops.rs ensure_rendered_outputs_current; crates/pray-core/src/verify/mod.rs verify_project and drift_project.
Notes: verify is dest managed spans versus Prayfile.lock. Drift runs those span checks and adds dest versus a fresh unpatched render (renderer_drift), grouped into sections. render --check only asks whether each dest file equals that fresh unpatched render, then stops on the first stale path with a render error. Combining them would hide that dest can match the lock and still differ from a fresh compose (unmarked notes, or a changed local source that dest has not re-embedded).

Outcome: partially supported as a metaphor.
Source: rfcs/0030-render-markers-ownership.md (verify is read-only; managed body versus ideal_checksum); README.md Planning, verifying, applying, and drift.
Notes: It is an integrity check of dest against the lock, closer to a compiled-output checksum than to a style linter. It does not print a unified dest diff. Orphan markers are warnings unless --strict. Other findings fail with exit code 6.

Outcome: partially supported as a metaphor.
Source: crates/pray-core/src/verify/format.rs format_drift_report; rfcs/0030-render-markers-ownership.md required report sections; crates/pray-cli/src/commands_verify.rs drift_semantic_command.
Notes: Default drift is a finding list in sections (Lockfile, Package, Managed span, Rendered file, Warnings), not a line diff. --semantic prints package version arrows and managed span counts only. pray plan is the command that previews what install would write. RFC renderer_drift text says on-disk matches lock but fresh render would change ideals. The implementation reports dest differs from fresh render even when dest also fails span checks.

Outcome: unsupported for pray render. Partially supported for RFC wording of pray render --check. Dry-run of materialization is pray plan.
Source: crates/pray-cli/src/commands_verify.rs render_command; crates/pray-cli/src/help_text.rs render and plan; rfcs/0030-render-markers-ownership.md section 44; rfcs/0034-unused-specified-surfaces.md; crates/pray-cli/src/commands_inspect.rs plan_command.
Notes: pray render without --check writes dest and Prayfile.lock. Help now names that write. --check writes nothing; it fails if dest bytes are not the fresh unpatched compose (exit class 5, render). RFC 0034 names plan and render --check as dry-run. Plan prints Using/Installing lines and would-be dest updates, including patched unmarked text. --check does not print a preview. Frozen install uses the same dest-equals-fresh test as --check.

Outcome: contested as operator confusion, unsupported as a product merge.
Source: rfc 0100 conformance bands (installer verify, renderer render --check, package manager drift); rfc 0002 CI (install --frozen and verify --strict); README.md example CI (verify --strict and drift).
Notes: CI may want lock honesty without dest identity. Unmarked dest notes keep verify clean and make render --check and drift renderer_drift fail. RFC 0002 frozen plus verify is dest identity plus lock honesty. README CI that runs drift on top of verify is dest versus fresh as well. Keep the verbs. Fix the help so the pair being compared is in the first sentence.

Help rewrite (2026-09-16): verify, drift, render, render --check, and plan help first sentences name the pair compared. Dry-run is plan. Rust and TypeScript render write dest and Prayfile.lock. Ruby render writes dest only.

Confirming checks: cargo test --offline -p pray-cli --test cli_ux (9 passed). bundle exec rspec spec/pray/cli_help_spec.rb from rubygems/pray-cli (7 examples, 0 failures). node --test dist/cli/help.test.js after npm run build (7 passed).

## Next

Do not fold render --check into drift in this pass. The overlap is real (both see dest versus fresh). The exit class and output shape differ (render error, one path, versus verify error, grouped findings). An alias can wait.

Pick one CI story: RFC 0002 frozen plus verify --strict, or README drift on top of verify. Do not collapse the verbs to paper over that choice.

## Source

usr/docs/issues/20260916180300_install-update-verify-local-compose.md
rfcs/0020-resolve-and-lock.md
rfcs/0030-render-markers-ownership.md
rfcs/0031-ownership-and-generated-output.md
rfcs/0034-unused-specified-surfaces.md
rfcs/0002-problem-and-positioning.md
rfcs/0100-conformance.md
crates/pray-cli/src/commands_verify.rs
crates/pray-cli/src/lockfile_ops.rs
crates/pray-core/src/verify/mod.rs
README.md
