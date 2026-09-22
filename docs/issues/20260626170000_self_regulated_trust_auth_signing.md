# Self-regulated trust auth and signing for publish

Build the trust stack for `pray publish` and registry ownership so package publishing feels closer to Rubygems/Git-style identity and signing.

## Desired capabilities
- register with email
- verify email by code
- add passkey
- add SSH key
- support SSH-agent-backed signing
- support web auth using email + passkey
- allow a registry owner to optionally disable email confirmation so any email + passkey is enough, while SSH key remains the second factor for CLI signing/login
- treat this as a building block for a self-regulated trust system

## Current gap
The repository currently has static registry publishing and install-time artifact verification, but no auth fixtures or implementation for email registration, passkeys, SSH keys, or SSH-agent signing.

## Proposed first slice
1. Add registry identity model and fixtures.
2. Add email registration + verification flow.
3. Add passkey-backed web login.
4. Add CLI session bootstrap.
5. Add SSH key enrollment.
6. Add SSH-agent-backed signing for publish.
7. Add end-to-end fixtures that prove publish and install through a trusted registry owner path.

## E2E fixture matrix
- web registration page
- email verification callback/code entry
- passkey login
- SSH key enrollment page
- CLI publish with SSH-agent signature
- served package page showing signer / trust state
- multiple clients installing from the same distribution point

## Acceptance criteria
- the auth and signing flow is covered by a reproducible fixture set
- publish can prove signer identity against registered credentials
- install can consume the published registry without manual fixture hacks
- the web surface shows the trust metadata needed for review and audit
