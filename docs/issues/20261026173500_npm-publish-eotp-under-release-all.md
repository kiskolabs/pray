# npm publish EOTP under release-all

## Participants

Andrei Makarov

## Decisions

Stop piping npm publish through tee so browser/device 2FA can finish. After a failed publish, wait for the registry version (covers E409 previously staged before GET is 200). JSR publish uses --allow-dirty so release-script edits do not abort resume.

## Effects

PRAY_RELEASE_YES=1 make release-all failed at npm publish pray-core@1.24.0 with EOTP while tee stole the TTY. After restoring bare npm publish, browser auth worked, then E409 previously staged stopped the run instead of skipping. pray-core@1.24.0 is on npmjs; pray-cli@1.24.0 was still missing at that point.

## Next

Resume PRAY_RELEASE_YES=1 make release-all. pray-core should skip; continue with pray-cli, JSR, and RubyGems.

## Source

scripts/release/npm.sh
scripts/release/README.md
