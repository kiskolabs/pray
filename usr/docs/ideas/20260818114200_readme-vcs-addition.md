# README: pray as an addition to version control

## Participants

Andrei Makarov

## Decisions

Version control records the history of one working tree. Nested-project features pin whole repositories, directories, or files at a revision. They do not resolve version ranges, compose named fragments into one inference-facing file, reconstruct a managed span, or give the same recipe to consumers that use different VCS tools.

Bundler and Cargo already treat Git as a package source and still lock exact revisions. Prayfile.lock is the lock for inference input. The consumer VCS still reviews Prayfile, Prayfile.lock, and usually the rendered files.

Git and Mercurial are auto-detected for publish and sync. Subversion and CVS use the configured command backend only.

Do not claim that Git cannot pin shared files. Submodules pin a commit. Subtree copies a tree. The gap is composition and range resolution, not history.

Omit a comparison table. Omit any claim that pray replaces a VCS or that drift reduction is measured.

## Effects

README.md gained Why use this with Git, Mercurial, Subversion, or CVS after Why. The section leads with what the VCS records.

Checked primary docs on 2026-08-18: git-scm book Submodules, git-submodule, Mercurial subrepos help, SVN Book externals, Subversion 1.6 file externals, CVS modules, RubyGems Gemfile.lock guide, Cargo git dependencies.

## Next

None. Re-audit if that README section is rewritten.

## Source

Upstream: README.md Why; SPEC.md git sources; crates/pray-cli/src/revision_backend.rs; usr/docs/changelogs/20260630153000_vcs_backed_revisioning_and_remote_storage.md.

Primary docs:

https://git-scm.com/book/en/v2/Git-Tools-Submodules

https://git-scm.com/docs/git-submodule

https://www.mercurial-scm.org/help/topics/subrepos

https://svnbook.red-bean.com/en/1.8/svn.advanced.externals.html

https://subversion.apache.org/docs/release-notes/1.6.html

https://www.sourceware.org/sourceware/cvs-docs/cvs_18.html

https://guides.rubygems.org/gemfile-lock/

https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html

Downstream: README.md.
