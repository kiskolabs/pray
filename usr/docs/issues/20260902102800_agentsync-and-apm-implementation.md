# AgentSync and APM implementation notes

## Participants

Andrei Makarov

## Decisions

None yet. This note is from source of dallay/agentsync and microsoft/apm cloned 2026-09-02 (depth 1 of each default branch). Docs alone were not enough. Prayers must not ship secrets. Packages stay inert.

## Effects

AgentSync and APM solve different jobs. AgentSync fans one .agents/ tree out to many tool paths in the same repository. APM resolves packages, deploys primitives, and compiles instruction files into AGENTS.md. Prayfile already owns the second job. Steal safety and apply-path discipline from both. Do not steal symlink fan-out, MCP install, or named start/end markers as the compose identity.

### AgentSync (dallay/agentsync)

Core loop is Linker.sync in src/linker/apply.rs. Four declared sync types: symlink, symlink-contents, nested-glob, module-map. Destinations are relative. create_symlink in src/linker/symlinks.rs builds a relative link, probes dest with symlink_metadata so a broken link is not treated as a missing file, and backs up a regular file to dest.bak before replacing it.

Path safety is in src/linker/paths.rs. ensure_safe_destination rejects absolute paths, ParentDir, RootDir, and Prefix, then joins project_root and canonicalize of an existing ancestor. revalidate_path runs again immediately before create, write, rename, or unlink. revalidate_unlink_path does not canonicalize the final component, so a symlink whose target is outside the project can still be removed if the link itself sits inside the root.

symlink-contents versus a directory symlink is detected in src/skills_layout.rs and surfaced by doctor and status. That is the Stow folding mismatch: applying per-child links onto a directory link churns. Status in src/commands/status.rs emits a JSON contract: destination kind, expected source, child not symlink, incorrect link target, missing expected source.

Gitignore management in src/gitignore.rs rewrites a block between # START marker and # END marker. Line trim equality. Teams can opt out and commit destinations. Default is to ignore the generated links.

Plugins in src/plugins.rs are closer to Prayfile than the symlink loop is. The module comment says it does not invoke vendor CLIs or execute plugin source. apply requires plugins.lock.toml; missing lock tells the operator to run plugin add or plugin update first. Apply compares discovered content_sha256, skill ids, and MCP names to the lock and fails on drift. Git sources materialize from a local snapshot under .agents/.agentsync-plugin-sources; apply must not fetch. Test apply_requires_a_local_snapshot_for_locked_git_sources asserts that. copy_directory_without_symlinks refuses any symlink in the tree. ApplyTransaction backups then rolls back on failure.

compress_agents_md_content collapses blank lines and inline whitespace outside fences. That is churn. RFC 0030 render churn: minimal forbids it for inference input.

Skills install and update skip or refuse package-tree symlinks (tests/test_update_security.rs). Windows symlink failures name Developer Mode and a docs URL.

### Microsoft APM (microsoft/apm)

Python CLI under src/apm_cli. Compile default is distributed (agents_compiler._compile_agents_md). GitHub issue 1764 recorded that apply_managed_section existed but the default distributed writer bypassed it, so managed_section silently full-overwrote AGENTS.md. Current main routes distributed writes through _prepare_distributed_file then apply_managed_section. Lesson: one write gate. A helper that only the legacy path calls is a product defect.

apply_managed_section in compilation/managed_section.py is a pure function. Non-empty distinct markers. str.count must be 1 for each. End index must be after start. Else ManagedSectionError, no write. Matching is substring count, not full-line tokens. A marker that appears inside generated text would duplicate and fail, or a shorter marker could match a longer one. Pray full-line <!-- pray:id --> is stricter.

Root managed_section requires the file to exist. Distributed new files are wrapped with _new_managed_section_content. Orphans that still contain those markers are retained; only files bearing AGENTS_MD_GENERATED_MARKER are auto-deleted on --clean. Nested Git repositories and linked worktrees are skipped with a warning (run compile from that root).

Install deploy uses BaseIntegrator.validate_deploy_path: no .., must start with a known integration prefix, must resolve under project_root. Discovery rejects primitive-tree symlinks. is_content_identical_to_source reads with O_NOFOLLOW so a dest cannot be swapped to a symlink between stat and read. Collision: adopt if bytes already match expected deploy form (raw or LF-normalized); otherwise skip unless force. Cleanup in integration/cleanup.py deletes stale deployed_files only if the on-disk hash still matches the lock; user-edited dests are skipped.

Dedup: if instructions already live under .github/instructions/ or the target rules dir, compile omits them from AGENTS.md so Copilot does not read the same text twice. Targets that only read AGENTS.md must not hit that skip (issue 1678). --force-instructions opt out.

Footer text says the section or file was generated by APM CLI and names apm compile. RFC 0031 user-facing preamble must not mention implementation details. source_attribution default false in schema; distributed_compiler still emits AGENTS_MD_GENERATED_MARKER on every file so orphan cleanup can tell generated from hand-authored.

Constitution injection in _prepare_distributed_file swallows exceptions at debug. Do not copy silent skip.

## Next

From AgentSync apply: revalidate immediately before mutate; unlink without following the link target; detect directory-link versus leaf-link mismatch for tree: destinations; plugin apply is frozen (lock plus local snapshot, hash check, no fetch, no execute, no package symlinks, transactional rollback). Status JSON is a cousin of pray verify --strict, not a replacement.

From AgentSync do not copy: symlink as the materialize strategy; dest.bak replacement of unmanaged files; AGENTS.md whitespace compression; default gitignore of rendered destinations (Prayfile usually commits them).

From APM apply: one write function for every compile path; fail closed on marker mistakes; O_NOFOLLOW when comparing dest to expected bytes; hash-gated prune that refuses to delete user-edited exclusive files; skip nested extra git roots; do not emit the same instruction body into two harness files unless the operator asks.

From APM do not copy: substring-named start/end as compose identity; generated-by footer that names the CLI; MCP in the same install; prefix allow-list that would block exclusive file: under a home root; install-then-audit as CI.

Map onto the home backlog: B6b refuse-clobber is AgentSync backup inverted and APM adopt-if-identical. Exclusive file: prune needs APM cleanup's hash gate. tree: should stay leaf copy (AgentSync symlink-contents), never fold a directory.

## Source

dallay/agentsync src/linker/apply.rs process_target and resolve_source_path
src/linker/symlinks.rs create_symlink handle_existing_symlink backup_existing_destination
src/linker/paths.rs ensure_safe_destination revalidate_path revalidate_unlink_path
src/gitignore.rs managed_markers remove_managed_section
src/plugins.rs apply copy_directory_without_symlinks ApplyTransaction
src/commands/status.rs StatusIssueKind
src/skills_layout.rs
tests apply_requires_a_local_snapshot_for_locked_git_sources
microsoft/apm src/apm_cli/compilation/managed_section.py
src/apm_cli/compilation/agents_compiler.py _compile_distributed _prepare_distributed_file _write_output_file_with_config
src/apm_cli/compilation/distributed_compiler.py AGENTS_MD_GENERATED_MARKER _cleanup_orphaned_files
src/apm_cli/integration/base_integrator.py is_content_identical_to_source validate_deploy_path O_NOFOLLOW
src/apm_cli/integration/cleanup.py hash-gated delete
src/apm_cli/primitives/discovery.py symlink reject
GitHub issue 1764 managed_section ignored on distributed compile
RFC 0030 churn, RFC 0031 preamble, RFC 0050 no package symlinks
usr/docs/issues/20260902102200_file-and-fragment-distribution-survey.md
