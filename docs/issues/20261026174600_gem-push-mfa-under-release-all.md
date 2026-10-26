# gem push MFA under release-all

## Participants

Andrei Makarov

## Decisions

Run gem push on the real terminal. A pipe through tee block-buffers the OTP or WebAuthn prompt, so make release-all looks stuck after the gem push line. After a failed push, skip when RubyGems already lists that version.

## Effects

PRAY_RELEASE_YES=1 make release-all reached gem push pkg/pray-cli-1.24.0.gem and produced no further output until the run was interrupted. The gem file was built. The push did not finish.

## Next

Resume PRAY_RELEASE_YES=1 make release-all. crates.io, npm, and jsr.io versions that are already published should skip. gem push should show the RubyGems MFA prompt. Optional: GEM_HOST_OTP_CODE for a one-time code.

## Source

scripts/release/rubygems.sh
scripts/release/README.md
docs/issues/20261026173500_npm-publish-eotp-under-release-all.md
