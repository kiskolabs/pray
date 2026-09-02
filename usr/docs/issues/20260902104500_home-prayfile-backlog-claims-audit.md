# Home Prayfile backlog claims audit

## Participants

Andrei Makarov

## Decisions

None yet. This note fact-checks the B0-B13 backlog in usr/docs/issues/20260902100800_home-prayfile-and-marker-dialects.md against current RFCs, crates, lock schema, and the three prior-art notes. It does not claim an RFC id and does not change product behavior.

Scope is the home-root, compose-marker, and file-or-fragment distribution thread. Production-readiness network slices and the 2026-09-01 auth or unpack audit residuals stay on their own notes.

## Effects

### Already true, do not re-specify as new work

Exclusive file: is unmarked, substitutes UTF-8 symbols, copies non-UTF-8 as bytes, and does not add an agent header. RFC 0010 file: rules already say that. tree: copies listed folder leaves into a real directory and does not delete undeclared siblings. Destination strings must be repository-relative; absolute paths and parent traversal fail in ProjectRelativePath. pray --path and PRAY_PATH select the project root. The CLI does not walk parents for a Prayfile. Opaque HTML comment markers are hardcoded in Rust render_compose, render_patch, verify parse_marker, and in the TypeScript marker parser. Packages stay inert. Symbols are explicit maps. require-signed packages already exist as trust policy. pray plan already exists as dry-run. Destination DSL already names compose, tree, and file: paths, so filesystem auto-detect of tool folders is not present.

### Publication-blocking errors in the prior backlog prose

Tilde is not rejected. A destination starting with ~ is a Normal path component. file: "~/.zshrc" writes a literal ~ directory under the selected root. Shell tilde expansion does not run. Operator notes must say file: ".zshrc" under pray --path HOME, not ~/.zshrc.

RFC 0011 export kinds that match no destination role live in section 23, not section 25. Section 25 is payload rules (allowed contents, no package execution).

Binary exports do not currently fail compose. RFC 0108 is Experimental and lists a binary-file-fails fixture that is not the product contract. should_inline_export skips every non-fragment kind, so a file export is omitted from compose rather than rejected as binary. Exclusive file: already copies non-UTF-8 bytes.

RFC 0030 "paths must stay inside the repository unless explicitly allowed" is a local-embed rule in section 43, not a home-root rule. Home as project root is a different mechanism: the whole tree is the project. Mixing those two sentences overstates a missing explicit-allow feature for home.

RFC 0031 says render deletes origin-tagged managed directories on package remove. Prayfile.lock has no origin tag and no provisioned-path array. apply writes planned files and does not prune dropped exclusive file: or tree: leaves.

### Material qualifications

RFC 0030 section 44 lists four render modes: managed, check, local, vendor. The parser rejects any mode other than managed. pray vendor is a CLI that copies packages into .pray/vendor; it is not render mode :vendor. B7 named only local.

Plan output does not list every write path. Sibling provisioned files under one parent are grouped as folder would be written (N files). provisioned_change_status compares destination bytes to the package source file, not to expected_provisioned_bytes after symbol substitution.

fs::write follows a destination that is already a symlink. B13 "symlink destinations fail" is the wanted contract, not current behavior. Package archives already reject symlink members.

spec.adapters on the prayspec is a string map that is parsed and never loaded. RFC 0030 section 46 describes separate TOML target adapter files that are also not loaded. Destination DSL already replaces that path-mapping job. Do not load adapters to spell markers.

section_markers is parsed onto RenderPolicy and never read by render or verify. line_endings is parsed; writes do not switch on that field; hashing always normalizes.

Compose of .zshrc or .json is not a type error. Any compose destination receives HTML comments and, when render.header is true, the Agent context banner.

Marker dialect RFC numbering: ids/0032 is unclaimed and sits in the 0030-0039 render band. 0114 is also free and is the wrong band. Claim 0032 only if dialects are actually scheduled. Dialects are optional for a useful home Prayfile.

Chezmoi modify_, Copier three-way merge, AgentSync symlink fan-out, sx SessionStart hooks, and APM named start/end markers remain correctly out of scope. APM issue 1764 (managed_section bypassed on distributed compile) is a neighbor warning: one write helper that only some paths call is a defect. Pray compose and provisioned already share one materialize caller; the remaining split is two write helpers that must share refuse-clobber and symlink checks.

### Claims ledger

Home issue: exclusive file: and tree: already materialize non-Markdown. Source: RFC 0010 file: rules, render_provisioned.rs. Outcome: supported.

Home issue: compose hardcodes HTML comments. Source: render_compose.rs, verify/mod.rs parse_marker, npmjs/pray-cli/src/verify/markers.ts. Outcome: supported.

Home issue: tilde expansion is rejected. Source: paths.rs ProjectRelativePath. Outcome: unsupported. Tilde is accepted as a relative name.

Home issue: render mode local specified and rejected. Source: RFC 0030 section 44, manifest_validate.rs. Outcome: partially supported. check and vendor are specified the same way and also rejected.

Home issue: RFC 0030 local explicit-allow unimplemented. Source: RFC 0030 section 43. Outcome: supported for local embeds. Not a home-root blocker.

Home issue: binary exports already fail compose under RFC 0108. Source: RFC 0108 Experimental fixtures, should_inline_export. Outcome: outdated relative to the shipped CLI.

Home issue: spec.adapters parsed and unused. Source: package_spec.rs. Outcome: supported.

Home issue: RFC 0011 lists unmatched export kinds. Source: RFC 0011 section 23, destination.rs export_kind_matches_role, render_provisioned collect_selected_export_files default arm. Outcome: supported. Section number in the original note was incomplete.

Home issue: exclusive file: overwrites. Source: render_provisioned write_provisioned_file fs::write. Outcome: supported.

Home issue: plan must list every path. Source: apply_report.rs grouped_provisioned_lines. Outcome: unsupported as current behavior. It is a wanted contract.

Home issue: symlink destinations fail. Source: fs::write in render_write.rs and render_provisioned.rs. Outcome: unsupported as current behavior.

Survey: exclusive file: needs a lock ledger for prune. Source: schema/lockfile.schema.json has managed_span only; RFC 0031 remove step 4. Outcome: supported gap.

Survey: Copier three-way merge rejected. Source: RFC 0108, RFC 0031 golden rule 4. Outcome: supported.

APM: fail closed on marker mistakes. Source: current parse_marker requires a full-line HTML token and an opaque id. Outcome: supported for HTML; dialects must not weaken this to substring start/end.

AgentSync: revalidate immediately before mutate; unlink without following. Source: not present in pray dest writes. Outcome: supported gap, same family as O_NOFOLLOW dest compare.

Dotfile prior art: native include plus unmarked file:. Source: yadm alternate docs cited in the 20260902101500 note. Outcome: supported as the home splice. Not re-fetched in this audit pass.

### Corrected backlog

Order is dependency. Later items assume earlier contracts. Home unmarked file: plus destination-native include can be documented after B0 and must not be recommended for real home trees until B6b and B15 land.

P0 Documentation honesty.

B0 Document two destination modes in RFC 0010 and README. file: and tree: own unmarked whole paths. compose splices spans only where a shipped wrapper is a legal comment. Today that wrapper is HTML comments only. Packages stay format-agnostic. State that compose of .zshrc or .json is currently legal and host-invalid. State that tree: copies leaves and leaves undeclared siblings. No new behavior.

P1 Safety before any operator note that uses HOME as --path.

B6b Refuse-clobber. Exclusive file: and tree: MUST fail when the destination exists, is not already the reconstructed expected bytes (expected_provisioned_bytes, including symbols), and is not already a managed path from the previous lock. Current write overwrites. Opt-in replace of an already-managed path stays apply. Do not copy dest.bak of unmanaged files.

B15 Destination metadata without following. Before read or write, use symlink_metadata. If the dest path is a symlink, fail. Compare expected bytes without following a replaced dest. Re-check immediately before the mutate. Package symlink reject stays.

B14 Provisioned path ledger in Prayfile.lock. Each exclusive file: and each tree: leaf records path, content hash, package, and export. Verify can name a missing path from the lock. This is the missing half of managed_span for unmarked files.

B16 Hash-gated prune. On package remove or export drop, delete a provisioned dest only if on-disk hash still matches the last locked expected hash. User-edited dests stay. Implements RFC 0031 remove step 4 without origin comments in the file. Depends on B14.

B13 Plan and home-root operator contract. Plan MUST print every path that would be written; stop grouping siblings when the audience is a home tree. Plan change detection MUST use reconstructed expected bytes. Path containment stays relative to the selected root. Provisioned scripts do not gain execute bits. Prefer require-signed packages in the operator note; that flag already exists. After B0, B6b, and B15, an operator note may show pray --path HOME with file: ".zshrc" and a human file that sources that path. No tilde in destination strings. No parent-walk. Note that Prayfile.lock and .pray live in that root.

B6 is that operator note. It is not a code feature beyond --path, which already works.

P2 Specified-but-dead surfaces. Independent of home.

B7 Strike or implement. spec.adapters and RFC 0030 section 46 adapter TOML: remove the claim or specify path mapping only, never marker spelling. Export kinds template, command, rule, asset, bundle: implement against tree or file, or drop them from RFC 0011 section 23 supported list. Render modes check, local, and vendor: strike from RFC 0030 or map check to pray render --check / plan, vendor to the vendor command, and local to an explicit directory mapping that is still inside the project root. section_markers and line_endings: gate writes or drop from the parsed policy. RFC 0031 origin-tagged directory delete: replace with B14 and B16 rather than tagging dest files.

P3 Compose of non-Markdown, still optional for home.

B8 Stabilize RFC 0108 file-as-fragment. Compose of a file export remains marked. Exclusive file: remains unmarked. Polyglot CLIs match Rust fixtures before Stable.

B3 Per-destination header. RFC 0108 already flags project-wide RenderPolicy.header. compose AGENTS.md may keep the Agent context banner. compose of any other dest MUST be able to set header: false. Header text must not mention .agents when the destination is not an inference root.

B4 Fail closed. Compose of JSON, binary, or an unknown type without an explicit markers: override is a render error that names file: as the unmarked path. Do not coerce. This is new product behavior, not current RFC 0108.

B5 Shebang. If the destination starts with #!, the first span and the ignore marker MUST NOT replace that line. Needed if hash-wrapper compose of scripts ships. Not needed for exclusive file: of a script.

B1 Marker dialect RFC. Optional. Claim 0032 if comment-friendly non-Markdown compose is still wanted after native include. Payload stays opaque pray:id. Wrappers are a closed table. Extensionless names have no default. Lock the wrapper per span. No package-provided parser. Do not encode package names into ids.

B2 Wire dialects through render, patch, verify, and format in Rust, then Ruby and TypeScript. Shared fixtures. Depends on B1, B3, B4.

B11 Draft reserved RFC 0110 marker id stability. Independent of dialects. Land before claiming equivalent renders are byte-stable across wrapper or grouping changes.

B12 Conformance. RFC 0100 stays Experimental until packs exist. Grow one pack per shipped wrapper and one home-root file: fixture. Coverage follows spec/README.md.

P4 Defer until native include is tried and found insufficient.

B9 Named slots inside exclusive file:. Unmarked splice. Separate RFC. Reconstruct from lock. Do not read live destination stdin.

B10 Sidecar spans. Second identity channel. Experimental only after B1-B4 and B9 are honestly specified.

### Still out of scope

User-defined file types, package-shipped marker plugins, destination paths starting with / (tilde as a name should be rejected in the same pass as B6b docs), Bundler-style walk-up into HOME, treating pray as a chezmoi replacement, modify_ scripts, encryption of secrets into prayers, bootstrap or onChange execution, host-probed templates, symlink farms, tree folding, --adopt, dest.bak of unmanaged files, default gitignore of rendered destinations, MCP in install, filesystem auto-detect of compose targets, APM named start/end as compose identity, substring marker matching, generated-by footers that name the CLI, install-then-verify as the CI pattern, transactional apply rollback as a home blocker, skipping nested git roots as a default (warn later if tree: would write into an extra git root).

## Next

Later pass: P2 B7 shipped as RFC 0034. P3 B8 B3 B4 shipped; RFC 0108 is Stable with matching Rust, Ruby, and TypeScript fixtures. Home as --path is a normal project folder. / as --path is allowed and not recommended. Do not claim ids/0032. B5 shebang, B11 RFC 0110, and B12 conformance packs stay later. RFC 0033 stays Experimental until remaining Windows no-follow checks.

Update the original home issue Next to point here so B0-B13 numbering is not copied as if already verified.

## Source

usr/docs/issues/20260902100800_home-prayfile-and-marker-dialects.md
usr/docs/issues/20260902101500_dotfile-manager-prior-art.md
usr/docs/issues/20260902102200_file-and-fragment-distribution-survey.md
usr/docs/issues/20260902102800_agentsync-and-apm-implementation.md
RFC 0010 file: rules, RFC 0011 sections 23 and 25, RFC 0030 sections 41 43 44 46, RFC 0031 remove steps, RFC 0040 PRAY_PATH, RFC 0050 package symlinks, RFC 0100 Experimental, RFC 0108 Stable, reserved RFC 0110
schema/lockfile.schema.json
crates/pray-core/src/paths.rs ProjectRelativePath
crates/pray-core/src/project_context.rs
crates/pray-core/src/manifest_validate.rs
crates/pray-core/src/render_compose.rs should_inline_export
crates/pray-core/src/render_provisioned.rs
crates/pray-core/src/render_write.rs
crates/pray-core/src/destination.rs export_kind_matches_role
crates/pray-core/src/package_spec.rs adapters
crates/pray-core/src/verify/mod.rs parse_marker
crates/pray-cli/src/apply_report.rs grouped_provisioned_lines provisioned_change_status
rfcs/ids/ (0032 absent, 0110 reserved, 0113 last claimed in the 0100 band)
