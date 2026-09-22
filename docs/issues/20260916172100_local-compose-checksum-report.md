# Local compose sources stay stale after install

## Participants

Andrei Makarov

## Decisions

A compose local such as pray ".agents/project.md" is a managed span. package is local. export is the path. source_checksum is sha256 of the file bytes. install, update, and plan re-embed that file. They print the path and how the checksum was checked.

pray install is the command after a local edit. pray update is not required to refresh checksums. outdated lists local checksum drift under Outdated local files.

The span record lives in Prayfile.lock managed_span. That is the verify and patch record, not a version-locked package. RFC 0030 still says local files are not package dependencies and are not version locked. RFC 0031 already re-embeds listed .agents files into their spans.

First install after this change takes leading fresh spans that dest does not yet have, so unmarked stale local bodies are dropped and the marked span is inserted before the first shared package span. Notes that sat unmarked in that leading region can be dropped once.

## Effects

Before this change, local compose bodies were unmarked dest text. render_patch kept that text. Changing .agents/project.md then running install, update, plan, or outdated left AGENTS.md and the lockfile looking unchanged. No checksum line named the local file.

After this change, a same-length local edit updates dest, lock source_checksum, and the install report (Updating path (sha256:... was sha256:...)). A second install with the same bytes prints Using path (sha256:... checked). The footer counts local files apart from packages.

Confirming checks in this pass: cargo test --offline -p pray-cli --test install_local (4 passed). cargo test --offline -p pray-cli --bin pray summary_ (2 passed). cargo test --offline -p pray-cli --test install_drift (5 passed). cargo test --offline -p pray-core --lib inserts_leading (1 passed). cargo test --workspace --offline (exit 0). cargo clippy --workspace --all-targets --offline -- -D warnings (exit 0). cargo fmt --all -- --check (exit 0). bundle exec rspec spec/pray/destination_render_spec.rb spec/pray/render_spec.rb spec/pray/verify_position_spec.rb spec/pray/install_spec.rb spec/pray/provisioned_dest_spec.rb spec/pray/update_destinations_spec.rb spec/pray/conformance_fixtures_spec.rb (51 examples, 0 failures). npm test in npmjs/pray-cli (202 pass, 0 fail). npm run lint in npmjs/pray-cli (biome, no circular deps). make loc-check (152 warnings, 0 failures).

## Next

Ships as 1.17.0. See usr/docs/issues/20260916223500_prepare-1-17-0-release.md.

Reconcile RFC 0030 section 43 (hashes may live in .pray/state.json, not Prayfile.lock) and RFC 0108 (local embeds stay unmarked) with RFC 0031 spans and this lockfile managed_span record.

Ruby and TypeScript wrap locals and patch dest the same way. Bundler-style Using/Installing/Updating checksum lines are on the Rust, Ruby, and TypeScript CLIs.

## Source

Reported: compose "AGENTS.md" do pray ".agents/project.md" end. After install, edit .agents/project.md. pray install printed package Using lines, Prayfile.lock unchanged, AGENTS.md unchanged, then Install complete. 19 packages, lockfile changed. pray update, plan, and outdated did not name the local file or a checksum.

rfcs/0030-render-markers-ownership.md
rfcs/0031-ownership-and-generated-output.md
rfcs/0108-file-as-fragment.md
crates/pray-cli/tests/install_local.rs
crates/pray-cli/src/apply_report_lines.rs
crates/pray-core/src/render_span.rs
crates/pray-core/src/render_patch.rs
usr/docs/changelogs/20260916172100_local-compose-checksum-report.md
