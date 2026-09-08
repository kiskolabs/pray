# Install git catalog refresh for new packages

## Participants

Andrei Makarov

## Decisions

pray install keeps a locked git revision when already resolved packages still work. When a newly declared package is absent from that catalog, install refreshes the git source the same way it already did for a version that the locked catalog cannot satisfy.

If the package is still missing after that refresh, the error names the locked revision and tells the operator to run pray update. It does not tell them to check the package name first.

plan --remote still previews from a refreshed catalog. apply still re-resolves from the lock instead of consuming that preview. After this change, apply uses the same install fallback, so a newly declared catalog package can resolve. Existing-package content updates still need pray update.

Do not hand-edit Prayfile.lock revision.

## Effects

Rust, Ruby, and TypeScript CLIs treat not found in distribution and missing v1/packages metadata as git refresh candidates. Git catalog misses name the locked revision and pray update.

## Next

None.

## Source

usr/docs/changelogs/20260908120000_install-git-catalog-refresh.md
