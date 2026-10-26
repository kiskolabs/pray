# npm publish EOTP under release-all

## Participants

Andrei Makarov

## Decisions

Stop piping npm publish through tee. That change from the skip-if-published work caused EOTP under make release-all. Restore a direct terminal npm publish like before. Detect already-published races with a registry version check after a failed publish. Accept NPM_OTP for authenticator 2FA.

## Effects

PRAY_RELEASE_YES=1 make release-all failed at npm publish pray-core@1.24.0 with EOTP. Earlier successful releases used bare npm publish. Keeping /dev/tty on stdin while still piping stdout through tee was not enough; npm still exited with EOTP.

## Next

Resume with an authenticator code: NPM_OTP=<code> PRAY_RELEASE_YES=1 make release-all. Or publish once by hand from npmjs/pray-core then resume so skips cover the rest.

## Source

scripts/release/npm.sh
scripts/release/README.md
