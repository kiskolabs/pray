# RFC 0051: Registry authentication delivery and enrollment

- Feature Name: registry-authentication
- Type: Standards Track
- Status: Proposed
- Created: 2026-09-01
- Author: Andrei Makarov
- Relates: RFC 0050, RFC 0060, RFC 0104
- Requires: RFC 0050, RFC 0060

## Summary

A distribution server that serves `auth.*` MUST deliver email verification codes out of band, mint a bearer session only after that proof, and enroll passkeys or SSH keys only for the session identity. Email-only session creation stays closed.

## Motivation

Registration used to return the verification secret in the HTTP body, and enrollment accepted an email plus attacker-chosen key material. Closing those paths without a delivery channel or a verified session left operators unable to enroll a first authenticator over the wire.

## Guide-level explanation

An operator serves a registry with `pray serve --root`. A caller posts `{"email":"alice@example.com"}` to `/v1/auth/register` and receives `email` and `verified` only. The plaintext code is appended to `.pray/verification-deliveries.jsonl` under that root, owner-read on Unix.

The operator gives Alice that code on a channel the registry does not serve. Alice posts email and code to `/v1/auth/verify` and receives a bearer token with `kind` `email`. She enrolls a passkey or SSH key on `/v1/auth/passkeys/enroll` or `/v1/auth/ssh-keys/enroll` with `Authorization: Bearer` and a body whose `email` matches the session.

`POST /v1/auth/session` with only an email returns 403. Later logins use passkey or SSH challenge response.

When `email_confirmation` is `disabled`, HTTP register creates a verified user and writes no code. The first authenticator is enrolled by an operator with filesystem access to the registry, not by a public email-only session.

## Reference-level explanation

Key words follow RFC 2119. HTTP paths match RFC 0104 optional `auth.*` methods.

`POST /v1/auth/register` MUST return 201 with `email` and `verified`. The JSON MUST NOT contain `verification_code` or the plaintext secret. When the store creates a code, the server MUST append one JSON line `email`, `code`, `created_at` (Unix seconds) to `{root}/.pray/verification-deliveries.jsonl` and MUST set that file to owner-only mode on Unix. Re-registering an existing email MUST NOT rotate or deliver a new code.

`POST /v1/auth/verify` with a matching unexpired code under the attempt ceiling MUST mark the email verified, mint a session of kind `email`, and return 200 with `email`, `verified` true, `token`, and `kind`. Stored codes and tokens remain hashes. Failures MUST use one error shape.

`POST /v1/auth/session` MUST return 403.

`POST /v1/auth/passkeys/enroll` and `POST /v1/auth/ssh-keys/enroll` MUST require `Authorization: Bearer` for a live session. The body `email` MUST equal the session email. The matching method MUST be enabled in `v1/trust.json`. Otherwise the server MUST return 403. Successful enroll binds the authenticator to the session email.

SSH-RPC `auth.*` methods use the same rules. A missing authorization parameter is the same as a missing HTTP header.

## Drawbacks

Operators must read a file on the server host. SMTP is not specified. Disabled confirmation has no public HTTP path to a first session.

## Rationale and alternatives

Returning the code in JSON recreates account takeover. Email-only `auth.session` recreates it after verify. A mailbox file under the registry root is the smallest out-of-band channel that does not add a mail transport. Hash-only storage plus bearer enroll is the smallest wire proof that enrollment is the same identity that verified.

## Prior art

WebAuthn and SSH CA enrollment both require an existing authenticated session before a new authenticator is bound.

## Unresolved questions

Whether a later RFC specifies SMTP or another delivery transport. Whether Disabled confirmation ever mints an HTTP session without an authenticator.

## Future possibilities

Credential rotation, revocation user experience, and authentication rate limits beyond the verify attempt ceiling.
