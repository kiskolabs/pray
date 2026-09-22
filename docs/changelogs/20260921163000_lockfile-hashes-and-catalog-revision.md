# Recalculate lockfile hashes and locked catalog revision check

## Participants

Andrei Makarov

## Decisions

After path sources moved under prayers/, committed Prayfile.lock hashes must match the current Prayfile. A locked git install keeps the pin in .pray-revision on the project catalog, not git HEAD. The project catalog is a file tree without a git directory.

## Effects

CI on main failed rust, ruby, and npm after the layout commit.

Ruby and npm publish tests opened packages/base on a copy of simple-project. That directory is now prayers/base.

Ruby install and parser specs still pinned the packages/ hash for simple-project. The parsed hash is sha256:52235a67dc445cbf8548de90999310ef42be3cafd81fe63a43a5ccd89ed047a1.

install_keeps_locked_git_revision_when_distribution_moves_forward ran git rev-parse HEAD in .pray/cache/git. That tree has no git objects. The check now reads .pray-revision.

A pray-core test now requires example and root lockfile hashes to match the current Prayfile, and pins the simple-project hash used by the Ruby client.

## Next

Confirm rust, ruby, and npm CI jobs on this branch.

## Source

usr/docs/issues/20260920112300_repository-layouts.md
usr/docs/changelogs/20260920112300_repository-layouts.md
usr/docs/changelogs/20260918213000_git-free-catalog-and-unshallow.md
.github/workflows/ci.yml run 35500653959
