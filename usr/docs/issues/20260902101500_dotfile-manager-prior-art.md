# Dotfile manager prior art for Prayfile

## Participants

Andrei Makarov

## Decisions

None yet. This note extracts features from Chezmoi, yadm, GNU Stow, and Nix home-manager against Prayfile contracts. Pray is not a secret store and must not ship confidential material in prayers. Packages stay inert. Host probes and environment capture stay out of render.

Recommended direction if work starts: keep exclusive file: and tree: as the unmarked whole-file path for home and for comment-hostile destinations; splice shared fragments with destination-native include rather than compose into live shell or JSON files; refuse to clobber an unmanaged exclusive destination; do not copy modify_ scripts, encryption, bootstrap, host-conditional templates, symlink farms, or tree folding.

## Effects

Line 27 of the home-Prayfile issue is mostly right and slightly too broad. Stow and home-manager own or generate whole destination files. Chezmoi defaults to whole files and also offers modify_ scripts and modify-templates that rewrite current destination bytes from stdin. yadm stores whole files in git and may emit a generated file from a template processor. None of the four leave an opaque in-band span id in the destination for later reconstruct-and-verify. Chezmoi modify_ is the only first-party patch-in-place contract among the four, and it does not require chezmoi comments in the destination.

That last point is the contract split. Pray compose reconstructs a marked span from lock plus packages. Chezmoi modify_ reads live destination bytes, runs a script or template, and writes the result. RFC 0031 golden rule 4 rejects three-way merge in v1. RFC 0011 section 25 forbids running package scripts. RFC 0050 forbids executing package code, capturing environment, and putting secrets in the lock. RFC 0002 already lists secret manager as a non-goal.

### Chezmoi

Primary: target types reference and manage-different-types-of-file user guide, templating user guide, encryption user guide. Accessed 2026-09-02.

Whole-file create, update, and delete is the default. Prefixes encode target kind in the source name: create_ writes only if missing; modify_ patches live dest; remove_ deletes; encrypted_ decrypts from source; executable_ private_ readonly_ set mode bits; .tmpl runs Go text/template plus sprig. Directories may be exact_, which deletes undeclared siblings. run_ scripts execute on apply, with once_ and onchange_ variants. symlink_ and mode = symlink make destinations point at source. Templates read hostname, os, and password-manager functions. apply --dry-run --verbose is the preview.

Adapt: dry-run before write is already pray plan. Source state versus destination state maps to package cache versus project destinations. Shared template fragments (.chezmoitemplates) map to package fragment exports, not to a new template engine. create_ is not a managed destination; optional local files already cover human-owned seed content that pray must not overwrite.

Reject: modify_ scripts (host execution, live stdin, no lock reconstruction). modify-template setValueAtPath as a patch of live JSON or YAML (same merge). encrypted_ and password-manager template functions (secrets). run_ scripts. exact_ and .chezmoiremove under home (deletes undeclared files). Host and os template branches inside packages (RFC 0050 avoid environment capture). symlink mode (RFC 0050 rejects symlinks in packages; verify needs destination bytes, not a link to cache). Mode bits on provisioned scripts unless a later RFC defines them.

setValueAtPath is useful only if the overlay is itself declared and locked and the whole destination is regenerated. That is exclusive file: of a generated document, not a live-file patch.

### yadm

Primary: Alternate Files and Templates pages, updated 2025-03-18, fetched 2026-09-02. Encryption and Bootstrap pages fetched 2026-09-02.

Alternates pick a whole file by scored conditions: os, arch, hostname, user, distro, class. Templates run awk, esh, j2cli, or envtpl over host data and environment variables. Encryption stores a gpg or openssl archive of matching paths. Bootstrap runs an operator-supplied executable after clone.

The useful pattern is not the suffix grammar. yadm's own alternate docs tell operators to keep one file when the format can branch, and otherwise to use a native include: a shared .gitconfig with path = .gitconfig.local, then an alternate only for the included file. That is destination-native include. The host parser owns the splice. The generated or selected file can stay unmarked.

Adapt: document native include as the home splice for comment-hostile files. Example shape: human-owned .zshrc sources a path that exclusive file: owns; human-owned .gitconfig includes a path that exclusive file: owns. This needs no marker dialect and no compose into .zshrc. Declared Prayfile groups and PRAY_ENV already select packages without uname. yadm class as local-only host config maps to optional local files and .pray/state.json, not to package bytes.

Reject: automatic hostname, os, user, distro selection inside packages. Template processors that execute shell (esh) or interpolate environment. Encryption of confidential files into a prayer or lock. Bootstrap execution. Shipping SSH keys, tokens, or private facts in packages even when encrypted.

### GNU Stow

Primary: stow(8) for GNU Stow 2.4.1 on Arch, dated 2024-09-08, fetched 2026-09-02. The gnu.org HTML manual returned 403; an older 1.3.3 HTML mirror was not used as the version of record.

Stow is a symlink farm. It creates relative symlinks from a target tree into a package tree. Tree folding replaces a directory with one symlink when the whole subtree belongs to one package. --no-folding disables that. If a plain unmanaged file already occupies the target, Stow records a conflict and refuses. --adopt moves that file into the package tree, which the man page warns is meant to alter the stow directory. Stow never deletes anything it does not own. --simulate is dry-run. --dotfiles maps a dot- prefix to a leading period.

Adapt: exclusive file: and tree: should refuse when the destination exists, is not already the reconstructed expected bytes, and is not already a pray-managed path. That is Stow conflict plus home-manager checkLinkTargets. tree: should copy leaves into a real directory and leave undeclared siblings (Stow --no-folding semantics). Two packages must not own one path (already specified). Dry-run is pray plan.

Reject: symlink farms. Tree folding of mixed directories such as .config. --adopt of home files into a prayer. Tilde expansion in resource files as a destination syntax.

### Nix home-manager

Primary: Home Manager Manual 24.11 (build versus activate, collision abort), home.file option page on nix-community.github.io fetched 2026-09-02, modules/files.nix and home-environment.nix on GitHub.

home.file declares whole files. Default force is false; force true silently replaces the target. Activation is a DAG. Entries before writeBoundary must be read-only checks. checkLinkTargets aborts on collision with unmanaged files. linkGeneration writes after the boundary. onChange runs shell after link. recursive false links a directory as one symlink; recursive true links leaves. DRY_RUN must log and not write.

Adapt: plan is the read-only side of writeBoundary. Exclusive writes should default to refuse-clobber, with an explicit later opt-in if a destination is already managed or byte-equal. tree: leaf copy matches recursive true, not a directory symlink.

Reject: onChange and home.activation scripts. Nix module option trees as a Prayfile surface. Generation rollback via a store; Prayfile.lock plus cache already cover reconstruct.

## Next

Fold these into the home-Prayfile backlog rather than opening a parallel product track.

B0 and B6 already cover documenting unmarked file: and tree: for home. Add an operator note for destination-native include: the human file stays unmarked and unspliced by pray; the included path is exclusive file:. That note can ship without marker dialects.

Add refuse-clobber for exclusive file: and tree: when the destination exists and is not the expected reconstructed bytes. Stow and home-manager treat that as the safe default. Current render_provisioned writes with fs::write and will overwrite. This matters under a home root and also in ordinary repositories.

Do not schedule modify_ equivalent, named live-file JSON patch, encryption, bootstrap, host-conditional package templates, or symlink destinations.

B1 marker dialects remain optional for compose into comment-friendly non-Markdown (TOML hash comments, shell hash comments) after per-destination header: false. They are not required to make a home Prayfile useful. Native include plus unmarked file: is the prior-art lesson for .zshrc, .gitconfig, and JSON.

B9 named slots inside exclusive file: must reconstruct from lock. The slot body is declared and checksummed. It must not read live destination stdin the way chezmoi modify_ does.

Keep B13: no secrets in prayers, no environment capture, no execute bit on inert scripts, no tilde or absolute destinations, prefer require-signed packages and plan before writes under home.

## Source

Chezmoi target types: https://www.chezmoi.io/reference/target-types/
Chezmoi manage different types of file: https://www.chezmoi.io/user-guide/manage-different-types-of-file/
Chezmoi templating: https://www.chezmoi.io/user-guide/templating/
Chezmoi encryption: https://www.chezmoi.io/user-guide/encryption/
yadm Alternate Files: https://yadm.io/docs/alternates (updated 2025-03-18)
yadm Templates: https://yadm.io/docs/templates (updated 2025-03-18)
yadm Encryption: https://yadm.io/docs/encryption
yadm Bootstrap: https://yadm.io/docs/bootstrap
GNU Stow 2.4.1 stow(8): https://man.archlinux.org/man/stow.8.txt (2024-09-08)
Home Manager Manual 24.11: https://home-manager.dev/manual/24.11/
home.file options: https://nix-community.github.io/home-manager/options.xhtml#opt-home.file
home-manager modules/files.nix and modules/home-environment.nix on GitHub nix-community/home-manager
RFC 0002 non-goals, RFC 0010 file: and symbols and groups, RFC 0011 sections 24-25, RFC 0030, RFC 0031 golden rule 4, RFC 0050, RFC 0108 future named slots
crates/pray-core render_provisioned.rs materialize_provisioned_exports
usr/docs/issues/20260902100800_home-prayfile-and-marker-dialects.md
usr/docs/issues/20260902102200_file-and-fragment-distribution-survey.md

Claims on the quoted line 27: whole-file ownership is supported for Stow and home-manager, and is the Chezmoi and yadm default (partially supported as a claim about all four equally). No in-band reconstruct markers: supported. Chezmoi modify_ without destination comments: supported by the target-types reference.
