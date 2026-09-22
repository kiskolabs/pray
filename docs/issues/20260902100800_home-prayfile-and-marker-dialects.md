# Home Prayfile and compose marker dialects

## Participants

Andrei Makarov

## Decisions

None yet. This note is analysis plus a proposed backlog. Recommended direction: keep exclusive file: and tree: as the unmarked whole-file path; do not add a new marker grammar per file type; split the existing opaque pray:id payload from a small closed comment-wrapper table; turn compose header off per destination; treat home as an explicit project root, not tilde paths in the manifest. Users cannot register a custom file type; packages must not ship parsers.

## Effects

Packages are not Markdown-only. RFC 0011 section 25 allows Markdown, TOML, JSON, YAML, plain text, templates, assets, and inert scripts. Exclusive file: destinations write UTF-8 after symbol substitution with no pray markers. tree: copies folder exports. Those two paths already materialize non-Markdown files when the project root is the directory that should receive them.

Compose is Markdown-shaped in practice. RFC 0030 and the README call HTML comments the Markdown canonical form. crates/pray-core render_compose, render_patch, verify parse_marker, and pray format all hardcode HTML pray comments. The default compose header is HTML ignore-comments plus an Agent context banner. RFC 0108 notes that RenderPolicy.header is one project-wide boolean.

A home directory as project root is mechanically possible without new path syntax. Destinations must be repository-relative. Absolute paths and parent traversal are rejected. A leading tilde is not expanded and is not rejected; file: "~/.zshrc" writes a literal ~ directory under the selected root. pray --path HOME or PRAY_PATH already selects the project root. A Prayfile at the home root with file: ".zshrc" or tree ".config" would write under home. Compose of .zshrc would also write, and would inject HTML comments and the Markdown header, which would break typical shell and config parsers.

Render modes check, local, and vendor are specified in RFC 0030 section 44. The parser rejects any render mode other than managed. pray vendor is a separate CLI command, not render mode :vendor.

RFC 0030 section 43 says local embed paths must stay inside the repository unless explicitly allowed. The explicit-allow path is not implemented. That rule is about local embeds, not about selecting HOME as the project root.

The CLI does not walk parent directories for a Prayfile. Invocation uses cwd, --path / PRAY_PATH, and --file-path / PRAY_FILE_PATH.

Chezmoi, yadm, GNU Stow, and Nix home-manager own or generate whole destination files, or transform current bytes through a source-side template or script. They do not leave opaque in-band span markers in the destination for later reconstruct-and-verify. Chezmoi modify_ scripts can patch an existing file; the destination need not contain chezmoi comments. That is a different contract from Pray compose spans. Prior-art extract: usr/docs/issues/20260902101500_dotfile-manager-prior-art.md. Native include plus unmarked file: is the useful home splice. Refuse-clobber of unmanaged exclusive destinations is the useful safety default. Do not copy modify_, encryption, bootstrap, host templates, or tree folding.

Comment-hostile formats cannot take in-band markers. JSON object syntax has no comments. Strict YAML and some INI consumers treat unknown comment shapes as data. RFC 0108 Experimental lists a binary-file-fails compose fixture; the shipped CLI skips non-fragment exports in compose and copies non-UTF-8 exclusive file: bytes as-is.

The scale problem is not missing wrappers for every extension. It is that compose identity (opaque id, pair on own lines, lock ideal checksum, unmarked text kept) is bound to one comment spelling, and the default header is bound to Markdown.

spec.adapters is parsed and never loaded. RFC 0011 section 23 lists template, command, rule, asset, and bundle export kinds that match no destination role.

## Next

Fact-checked order, corrected claims, and new items B14-B16 live in usr/docs/issues/20260902104500_home-prayfile-backlog-claims-audit.md. That note is the backlog of record. Do not copy the first-draft B0-B13 list from git history as if already verified.

Later pass: leading tilde dest strings are rejected. Compose of JSON, binary, or unknown types fails and names file:. Agent context banner defaults on AGENTS.md only. RFC 0108 is Stable. Home remains a normal --path root. / as --path is allowed and not recommended. Do not claim compose-into-home-dotfiles until marker dialects (B1 B2) ship. Do not add a user plugin for file types. Do not claim ids/0032 unless dialects are scheduled. B5 shebang stays off. B11 reserved RFC 0110 then B12 conformance packs.

## Source

Upstream: RFC 0002, RFC 0010, RFC 0011 sections 23 and 25, RFC 0030 sections 41 43 44 46, RFC 0031, RFC 0040 PRAY_PATH, RFC 0050 path traversal, RFC 0108 header and file-as-fragment, reserved RFC 0110, README render markers, crates/pray-core paths.rs ProjectRelativePath, project_context.rs, render_compose.rs, render_patch.rs, verify/mod.rs parse_marker, destination.rs export roles, package_spec.rs adapters field, manifest_validate.rs render mode, chezmoi target types and modify_ files, yadm alternates, GNU Stow symlink model.

Downstream: this issue backlog B0-B13 as first draft; usr/docs/issues/20260902101500_dotfile-manager-prior-art.md; usr/docs/issues/20260902102200_file-and-fragment-distribution-survey.md; usr/docs/issues/20260902102800_agentsync-and-apm-implementation.md; usr/docs/issues/20260902104500_home-prayfile-backlog-claims-audit.md.
