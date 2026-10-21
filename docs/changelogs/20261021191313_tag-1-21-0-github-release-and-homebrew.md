# Tag 1.21.0, GitHub Release, and Homebrew

## Participants

Andrei Makarov

## Decisions

Cut annotated tag v1.21.0 on main after language registries already published 1.21.0. Publish the GitHub Release from the 1.21.0 CHANGELOG section. Bump Homebrew to v1.21.0 and push amkisko/homebrew-tap main, including the earlier local bumps that had stayed unpushed after v1.16.0.

## Effects

Annotated tag v1.21.0 points at 9cca681. The tag is on GitHub and GitLab. GitHub Release https://github.com/kiskolabs/pray/releases/tag/v1.21.0 is the latest. Body is the 1.21.0 CHANGELOG section only. Name is v1.21.0.

make bump-homebrew wrote Formula/pray.rb to the v1.21.0 archive with sha256 80aaa9a451a9c81847665599354e7b580dc0033b61eba8d6246dc28ae7c682a4. git push of amkisko/homebrew-tap moved origin/main from 8e4e62b to f2c10e4. That range includes the previously local Bump pray to v1.17.0 and Bump pray to v1.19.0 commits, then Bump pray to v1.21.0. GitHub Formula/pray.rb now uses https://github.com/kiskolabs/pray/archive/refs/tags/v1.21.0.tar.gz.

crates.io, npmjs, and RubyGems already had pray-cli 1.21.0 before this pass. No registry republish from this pass.

Operator surface assessed: GitHub Release v1.21.0. Hierarchy is the tag title, then the CHANGELOG heading, then seven bullets covering hook-inherited Git variables, damaged Git cache refusal, published_at parsing, prayer source layout, lockfile hash recalculation, Rust 1.91 minimum, and render --check layout. The page is Latest, not a draft or prerelease. A person can read what shipped. Remaining for a human: brew upgrade amkisko/tap/pray on a machine that refreshes the tap.

## Next

Confirm brew upgrade amkisko/tap/pray installs 1.21.0.

## Source

CHANGELOG.md 1.21.0
docs/releasing.md
docs/issues/20261021180938_prepare-1-21-0-release.md
docs/changelogs/20261021180939_prepare-1-21-0-release.md
https://github.com/kiskolabs/pray/releases/tag/v1.21.0
https://github.com/amkisko/homebrew-tap/commit/f2c10e4
