# Out-of-band verification delivery and bearer enrollment

## Participants

Andrei Makarov

## Decisions

RFC 0051 specifies registry authentication: verification codes go to `{root}/.pray/verification-deliveries.jsonl`, verify mints an email session, enroll requires Authorization Bearer, and POST /v1/auth/session stays 403.

Disabled email confirmation still has no public HTTP path to a first session. The operator enrolls the first authenticator with filesystem access to the registry store.

## Effects

HTTP register writes the delivery file and omits the code from JSON. HTTP verify returns token and kind email. Passkey and SSH enroll succeed with a matching bearer session and return 403 without it or when the body email does not match.

## Next

SMTP or another delivery transport is an RFC 0051 unresolved question. Tag v1.9.2 after this lands.

## Source

RFC 0051
usr/docs/issues/20260901150000_diff-engineering-audit.md D-004
crates/pray-cli/src/server_auth.rs
crates/pray-cli/src/server_auth_delivery.rs
