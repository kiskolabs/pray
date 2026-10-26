# Tag 1.25.0, GitHub Release, Homebrew, and AUR

## Participants

Andrei Makarov

## Decisions

Cut annotated tag v1.25.0 on main after language registries already published 1.25.0. Publish the GitHub Release from the 1.25.0 CHANGELOG section. Rewrite existing GitHub Release notes with make release-github-sync. Bump Homebrew to v1.25.0 and push amkisko/homebrew-tap main. Refresh the AUR PKGBUILD sha256 from the tagged archive.

## Effects

Annotated tag v1.25.0 points at 2ab433f. The tag is on GitHub and GitLab. GitHub Release https://github.com/kiskolabs/pray/releases/tag/v1.25.0 is the latest. Body is the 1.25.0 CHANGELOG section only. Name is v1.25.0.

PRAY_RELEASE_YES=1 make release-github created the release. PRAY_RELEASE_YES=1 make release-github-sync rewrote notes for every existing GitHub Release from CHANGELOG.md. Versions without a GitHub Release stayed unpublished, including v1.22.0 through v1.24.0 (no tags or releases on the remote for those cuts).

make bump-homebrew wrote Formula/pray.rb to the v1.25.0 archive with sha256 52e4cc482775ab2f62db16a35d1f9b180781f015a0dfab544a350449a9cef9ce. git push of amkisko/homebrew-tap moved origin/main from f2c10e4 to 8901b85. GitHub Formula/pray.rb now uses https://github.com/kiskolabs/pray/archive/refs/tags/v1.25.0.tar.gz.

make bump-aur set packaging/aur/pray-cli/PKGBUILD sha256sums to the same digest. That PKGBUILD change is on main as 6c37e3f. The recipe is still not uploaded to AUR.

crates.io, npmjs (pray-core and pray-cli), jsr.io (@kiskolabs/pray-core and @kiskolabs/pray-cli), and RubyGems already had 1.25.0 before this pass. No registry republish from this pass.

Operator surface assessed: GitHub Release v1.25.0. Hierarchy is the tag title, then the CHANGELOG heading, then five bullets covering jsr.io pray-cli publish, pray-core positioning copy, Deno/Bun and parse runtime bounds, npm publish 2FA resume, and RubyGems MFA resume. The page is Latest, not a draft or prerelease. A person can read what shipped. Remaining for a human: brew upgrade amkisko/tap/pray on a machine that refreshes the tap; set jsr.io Settings description and runtime badges; adopt or upload the AUR package when ready.

## Next

Confirm brew upgrade amkisko/tap/pray installs 1.25.0.
Set jsr.io package Settings description and runtime compatibility for both packages.
Optionally create missing GitHub tags and Releases for v1.22.0 through v1.24.0 from CHANGELOG.md.
Copy packaging/aur/pray-cli into the AUR git package when ready to publish.

## Source

CHANGELOG.md 1.25.0
docs/releasing.md
docs/issues/20261026182000_prepare-1-25-0-release.md
docs/changelogs/20261026182000_prepare-1-25-0-release.md
https://github.com/kiskolabs/pray/releases/tag/v1.25.0
https://github.com/amkisko/homebrew-tap/commit/8901b85
