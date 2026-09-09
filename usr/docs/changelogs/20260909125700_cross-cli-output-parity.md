# Cross-CLI compose and manifest hash parity

## Participants

Vesa Vänskä
Andrei Makarov

## Decisions

Rust remains the reference for composed file bytes and canonical manifest JSON. Ruby and TypeScript follow that byte layout so `pray verify` and lockfile `manifest_hash` do not depend on which CLI ran install.

## Effects

Ruby ContentBuilder#finish now mutates the buffer with sub! so trailing blank lines collapse to one newline. Ruby emits symbols before render in canonical manifest JSON. TypeScript emits absent package locators and an absent target max_bytes as null. The TypeScript file locator still omits when absent, matching Rust skip_serializing_if.

Registry cache layout named in pull request 19 is already aligned on main and is not part of this change.

## Next

Pull request 19. Rebase conflicts were changelog-only; bullets live under Unreleased.

## Source

https://github.com/kiskolabs/pray/pull/19
CHANGELOG.md Unreleased
npmjs/pray-cli/CHANGELOG.md Unreleased
rubygems/pray-cli/CHANGELOG.md Unreleased
