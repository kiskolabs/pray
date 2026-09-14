# Cross-CLI compose and manifest hash parity

## Participants

Vesa Vänskä
Andrei Makarov

## Decisions

Rust remains the reference for composed file bytes and canonical manifest JSON. Ruby and TypeScript follow that byte layout so `pray verify` and lockfile `manifest_hash` do not depend on which CLI ran install.

## Effects

Ruby ContentBuilder#finish now mutates the buffer with sub! so trailing blank lines collapse to one newline. Ruby emits symbols before render in canonical manifest JSON. TypeScript emits absent package locators and an absent target max_bytes as null. The TypeScript file locator still omits when absent, matching Rust skip_serializing_if.

Registry cache layout named in pull request 19 is already aligned on main and is not part of this change. That pull request is merged.

## Next

Later pass 20260914140500: this work ships as 1.14.0. See usr/docs/issues/20260914140500_prepare-1-14-0-release.md.

## Source

https://github.com/kiskolabs/pray/pull/19
CHANGELOG.md 1.14.0
npmjs/pray-cli/CHANGELOG.md 1.14.0
rubygems/pray-cli/CHANGELOG.md 1.14.0
usr/docs/issues/20260914140500_prepare-1-14-0-release.md
