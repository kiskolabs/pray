# D-004 out-of-band verify and bearer enroll

## Participants

Andrei Makarov

## Decisions

Close D-004 on patch/fix-ruby-praypkg-unpack-cache. Do not restore email-only session issue. Specify the wire contract in RFC 0051.

## Effects

Operators receive verification codes in `.pray/verification-deliveries.jsonl`. Verify returns a bearer session. Enroll requires that session.

## Next

Publish 1.9.2 after review. SMTP delivery remains unspecified.

## Source

usr/docs/issues/20260901150000_diff-engineering-audit.md
usr/docs/changelogs/20260901164000_oob-verify-bearer-enroll.md
RFC 0051
