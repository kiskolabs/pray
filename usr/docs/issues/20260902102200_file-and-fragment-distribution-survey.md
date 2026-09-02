# File and fragment distribution survey

## Participants

Andrei Makarov

## Decisions

None yet. This note inventories known implementations that distribute whole files or splice fragments into destinations. It sits beside usr/docs/issues/20260902101500_dotfile-manager-prior-art.md, which already covers Chezmoi, yadm, GNU Stow, and Nix home-manager. Prayers must not ship secrets.

The survey is not a complete catalogue. It groups tools by contract so Prayfile can steal the right analog and skip the rest.

## Effects

Most tools in this space own whole files, symlink whole files, or merge structured documents. Very few reconstruct an opaque marked span from a version-range lock. Microsoft APM is the closest inference-input neighbor: it locks hashes, deploys files, and can compile many instruction primitives into AGENTS.md. Its managed section is one named HTML block, not per-export opaque ids that cite a lock record.

### Already covered

Chezmoi, yadm, GNU Stow, Nix home-manager. Whole-file ownership or live-file patch. No reconstruct-and-verify span markers. Details in the 20260902101500 note.

### More whole-file home and bootstrap tools

Chezmoi comparison table (fetched 2026-09-02): dotbot, rcm, vcsh, yadm, bare git.

dotbot (anishathalye/dotbot): YAML or JSON recipe. Link, create directory, shell, clean broken links. Destinations are symlinks. Plugins add brew, age, git-crypt. Whole files. Shell plugins execute. Reject execution and secrets plugins.

rcm (thoughtbot): rcup, mkrc, rcdn, lsrc. Source directory, usually ~/.dotfiles. Installs by symlink. host-HOSTNAME directories and tag-NAME directories select whole files. rcrc COPY_ALWAYS copies instead of linking. Default rcup prompts when a dest exists and does not match. Host and tag selection is like yadm alternates. Adapt: declared groups already cover tags. Reject: hostname directories inside a prayer.

vcsh (RichiH/vcsh): several Git repositories whose work tree is HOME. Files live in HOME, not as symlinks. myrepos (mr) batches clone and update. Whole files. No fragment splice. Analog of treating HOME as a git root, not of compose.

bare git: a Git repo with work tree HOME. Same whole-file contract as vcsh without multi-repo.

Other names that show up in the same class and were not fetched as primary this run: homesick or homeshick, Mackup, Pearl, fresh. Treat as unverified until a later pass. They are expected to be whole-file copy or symlink.

### Nested VCS, not fragment compose

README already records Git submodules, Git subtree, Mercurial subrepos, Subversion svn:externals including file externals since 1.6, and CVS modules. Those pin a whole tree or one whole file from another VCS identity. They do not compose a named export into a span inside AGENTS.md, and they do not resolve a version range across a package index.

### Scaffold update with three-way merge

Copier official updating docs: regenerate from old template, diff against the current project, apply that diff onto a new template generation. Conflicts become git-style markers or .rej files. skip_if_exists seeds once. cookiecutter generates once. cruft wraps cookiecutter and uses git apply -3, with a documented fallback to reject hunks.

RFC 0108 already rejected Copier three-way merge. RFC 0031 golden rule 4 stands. Do not adapt live merge. Copier check-update is a cousin of pray drift for template version, not for span checksums.

### Structured overlay, not Markdown spans

Kustomize: base resources plus strategic merge patches or RFC 6902 JSON Patch. The destination is a generated YAML stream. No in-band comment markers. Order and target selectors matter.

JSON Patch RFC 6902: add, remove, replace, move, copy, test on a JSON pointer. Chezmoi setValueAtPath is the same idea for a live file. For Prayfile, a locked overlay that fully regenerates a JSON document is exclusive file:. Patching live JSON is merge.

systemd.unit drop-ins: foo.service.d/*.conf merged in alphanumeric order after the main unit. The host parser owns the splice. Same family as shell profile.d, sysctl.d, sudoers.d, and git include. This is destination-native include, already recommended for home.

CUE unification, Jsonnet mixins, Dhall imports: generate or constrain structured config. They are languages, not lock-and-render of Markdown spans. Do not add a config language to Prayfile.

### Document include

mdBook {{#include}} inlines a file or an anchored slice into a chapter. AsciiDoc include:: does the same. RFC 0108 already cites both. Jekyll include and Docusaurus MDX import are the same pattern inside a doc build. The include directive lives in the destination source. Pray compose inverts that: the recipe in Prayfile names what is inlined, and the destination holds opaque markers that cite the lock.

### Shareable lint and editor config

ESLint shareable configs, RuboCop inherit, Stylelint extends, EditorConfig: a package or file that another config includes by name. Whole-file or structured merge of rules, not marked spans in a human document. Closest analog is destination-native include plus exclusive file:.

pre-commit: a manifest of hook repos locked by rev. It distributes executable hook environments, not text fragments into AGENTS.md. Execution is a Prayfile non-goal.

### Inference-input neighbors

Microsoft APM (microsoft/apm, docs at microsoft.github.io/apm, accessed 2026-09-02). Manifest apm.yml, lockfile apm.lock.yaml with resolved commit and SHA-256 content hashes, cache apm_modules/. Deploy copies primitives into harness directories. apm compile folds instruction primitives into AGENTS.md, CLAUDE.md, GEMINI.md, and per-harness rule trees. compilation.agents_md.mode managed_section replaces one block between start_marker and end_marker, default example HTML comments apm:start and apm:end. Markers must appear exactly once or compile fails. source_attribution is opt-in cosmetic origin comments. Default compile overwrites the whole root file. Transitive deps. MCP servers are in the same manifest. Security scan at install. apm audit replays deploy and diffs.

Adapt from APM only at the contract level: lock hashes, deploy whole files, optional one managed section, dry-run, refuse silent overwrite of hand-authored root files. Do not copy MCP install, auto-detect targets from which tool folders exist on the current machine (APM compile docs warn that committed output then tracks who last compiled), or a single named section that cannot map many package spans to lock records. Pray markers stay opaque ids. APM attribution comments are not reconstruct-and-verify.

AgentSync (dallay.github.io/agentsync, crate agentsync): one .agents/ tree, agentsync.toml, symlink fan-out to CLAUDE.md, AGENTS.md, Copilot paths, skill and command dirs. Same-repo sync, not a cross-repo version-range lock. Symlinks: Stow lesson, RFC 0050 rejects package symlinks, Windows and clone pain documented by rulesync. Reject as the materialize strategy.

rulesync (PyPI 1.0.0): one canonical rules.md, generate AGENTS.md, CLAUDE.md, .cursorrules, Copilot, Gemini, Windsurf, Aider, OpenCode. Dry-run and status. Same-repo fan-out with format shims. No package index or span checksums.

sx (sleuth-io/sx, blog 2026-06-05): sx.toml, resolver, client-native install of skills, rules, commands, hooks, MCP. Lockfile is per-user because resolution takes caller identity. SessionStart hook re-resolves on every Claude session. RFC 0002 forbids hidden self-update. Per-user lock is the opposite of a committed Prayfile.lock that two machines must share.

None of these four replace Prayfile. APM is the one that already has manifest, lock hashes, and compile into AGENTS.md. Prayfile still differs: version-range resolve, opaque per-span markers, reconstruct-and-verify, unmarked exclusive file:, no package execution, no MCP, no session hook.

What to learn from APM, against official schema, lockfile spec, compile, audit, and security pages accessed 2026-09-02.

Already the same bet: commit a lock next to the manifest; pin content hashes; keep a gitignored cache; CI frozen install plus a read-only check; reconstruct managed output rather than merge; keep human text outside managed bytes; prefer many small files over one giant compile (APM compilation.strategy default distributed).

Take as operator warnings, not new product shape.

Declare render targets in the manifest. APM compile docs say omitted targets auto-detect from which tool folders exist on this machine, so committed AGENTS.md and CLAUDE.md track who last compiled. Pray destination DSL already names compose, tree, and file: paths. Keep that explicit. Do not add filesystem auto-detect.

Lock a ledger of every deployed path plus a per-file hash. APM lockfile purpose list is reproducibility, integrity, prune of orphans, and inspection. Prayfile.lock already records managed spans with ideal checksums. Exclusive file: and tree: still need the same ledger so a dropped package can prune dest files and verify can name missing paths. RFC 0031 already says remove managed blocks and origin-tagged dirs on package remove.

Verify must be read-only. APM drift docs record a CI blind spot: apm install then apm audit restores deployed bytes first, so a hand-edit disappears before content-integrity runs. apm audit --ci now replays into a scratch tree. pray verify is already read-only. Do not document or implement install-then-verify as the CI pattern. Frozen install plus verify --strict stays the RFC 0002 job.

Fail closed on marker mistakes. APM managed_section requires the dest file to exist, both markers exactly once, start before end, distinct strings; otherwise ManagedSectionError, no silent overwrite. Pray unmatched markers are already invalid. Keep that. Do not adopt APM's one named pair (apm:start / apm:end). RFC 0030 needs an opaque id per span that cites the lock. One blob cannot checksum many packages.

Attribution comments default off. APM source_attribution is opt-in cosmetic origin, version, and generated-by footer. RFC 0030 already forbids duplicating graph, hashes, or provenance in the rendered file. Keep markers compact.

Install does not equal compile. APM install deploys primitives; apm compile writes AGENTS.md. Operators who only install get skills without the root context file. Pray install currently resolves, locks, and renders. Keep one user-facing apply path. Internal phases may stay split.

includes consent. APM audit warns when local content is deployed without an explicit includes field. That is the same leak class as putting private local files into a prayer. Keep local files out of the lock and out of publish.

Reject from APM: MCP and LSP in the same manifest; scripts and apm run (host execution, including ${{ secrets.KEY }} in the schema appendix); marketplace authoring as a product surface; hidden Unicode scan as a substitute for hash verify (optional later, not a contract); generated_at timestamps as identity; compile auto-detect; replacing opaque pray:id with named start/end; merging instruction conflicts by applyTo glob (RFC 0031: no three-way merge, package-versus-package compose collision fails).

### Patch series

quilt, StGit, git format-patch: ordered hunks applied to a tree. The unit is a diff, not a named export with an ideal checksum. Do not adapt as compose.

## Next

Keep B6 destination-native include. systemd drop-ins, git include, profile.d, and rcm COPY_ALWAYS are more evidence that the host parser should own the splice for comment-hostile files.

Keep B6b refuse-clobber. rcup -i, Stow conflict, home-manager force false, and APM skip of hand-authored root files without a generated marker all point the same way.

Do not add Copier merge, Kustomize JSON Patch of live files, AgentSync symlink fan-out, sx session hooks, or APM named start/end markers as a replacement for opaque pray:id pairs.

If an RFC later compares inference-input neighbors, cite APM compile managed_section and lockfile hashes as the nearest shipped analog, then state the span-id and no-execution gaps. Implementation notes: usr/docs/issues/20260902102800_agentsync-and-apm-implementation.md.

## Source

Chezmoi comparison table: https://www.chezmoi.io/comparison-table/
dotbot: https://github.com/anishathalye/dotbot
rcm rcup(1): http://thoughtbot.github.io/rcm/rcup.1.html
rcm rcm(7): https://thoughtbot.github.io/rcm/
vcsh: https://github.com/RichiH/vcsh
Copier updating: https://copier.readthedocs.io/en/stable/updating/
Kustomize JSON Patch example: https://github.com/kubernetes-sigs/kustomize/blob/master/examples/jsonpatch.md
JSON Patch RFC 6902: https://www.rfc-editor.org/rfc/rfc6902
systemd.unit drop-ins: https://www.freedesktop.org/software/systemd/man/systemd.unit
CUE configuration use case: https://cuelang.org/docs/concept/configuration-use-case/
mdBook include cited in RFC 0108
Microsoft APM: https://microsoft.github.io/apm/ and manifest schema https://microsoft.github.io/apm/reference/manifest-schema/ (lockfile section 8, compilation.agents_md section 6.2)
APM compile: https://microsoft.github.io/apm/reference/cli/compile/
APM security model: https://microsoft.github.io/apm/enterprise/security/
AgentSync getting started: https://dallay.github.io/agentsync/guides/getting-started/
rulesync: https://pypi.org/project/rulesync/
sx: https://github.com/sleuth-io/sx and https://sleuth-io.github.io/sx/2026/06/05/a-package-manager-for-ai-assets.html
RFC 0002, RFC 0031, RFC 0050, RFC 0108, README nested VCS section
usr/docs/issues/20260902101500_dotfile-manager-prior-art.md
usr/docs/issues/20260902100800_home-prayfile-and-marker-dialects.md

Claims: APM lock hashes and managed_section markers are supported by the manifest schema. Copier three-way update is supported by the official updating page. systemd drop-in merge is supported by systemd.unit. sx per-user lock and SessionStart hook are supported by the 2026-06-05 sx blog. AgentSync symlink model is supported by its getting-started guide. homesick, Mackup, Pearl, fresh were not verified this run.
