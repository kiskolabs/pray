# Deploying pray over SSH

Use this guide when pray client and pray server communicate only through SSH, without HTTP or a reverse proxy.

## Overview

1. Install `pray` on the server host.
2. Create a distribution root (for example `/var/lib/pray`).
3. Configure OpenSSH to run `pray serve --stdio` for the pray user.
4. Point consumer Prayfiles at `pray+ssh://pray@your-host`.

The server stores the same static layout as HTTP distribution (`v1/index.json`, `v1/packages/`, `v1/artifacts/`).

## OpenSSH configuration

```sshconfig
Match User pray
    ForceCommand /usr/bin/pray serve --stdio --root /var/lib/pray
    AllowTcpForwarding no
    X11Forwarding no
    PermitTTY no
```

Or as a subsystem:

```sshconfig
Subsystem pray /usr/bin/pray serve --stdio --root /var/lib/pray
```

Subsystem clients connect with:

```bash
ssh -s pray@your-host pray
```

## Publisher identity and push policy

SSH push authorization uses the connecting user's public key fingerprint, not a human label.

On the client or in the SSH session environment, set one of:

- `PRAY_SSH_USER_FINGERPRINT` (preferred)
- `SSH_USER_FINGERPRINT`
- `PRAY_SSH_PUBLISHER` (legacy alias)

Example:

```bash
export PRAY_SSH_USER_FINGERPRINT="$(ssh-keygen -lf ~/.ssh/id_ed25519.pub | awk '{print $2}')"
pray publish --server pray+ssh://pray@your-host
```

Optional push policy file at `v1/ssh_publishers.json`:

```json
{
  "publishers": [
    {
      "fingerprint": "SHA256:abcdef...",
      "id": "team-ci",
      "push": true
    }
  ]
}
```

When this file is present, `artifact.put` and `sync.push` over stdio require the active SSH user fingerprint to match an entry with `"push": true`. When the file is absent, push is open for private standalone hosts.

Consumers can import publisher fingerprints (and, for `pray+ssh` sources, the server host key) into local client trust policy:

```bash
pray trust import-registry pray+ssh://pray@your-host
```

Use `--match-prefix` to scope the rule, or `--no-host-key` to skip host key pinning.

## Consumer Prayfile

```manifest
source "team", "pray+ssh://pray@your-host"
agent "sample/base", "~> 1.0", source: :team
```

Optional port and root path hint:

```manifest
source "team", "pray+ssh://pray@your-host:2222/var/lib/pray"
```

`Prayfile.lock` may record `host_key_fingerprint` for `pray_ssh` sources and `signer_fingerprint` on locked packages when registry metadata includes SSH fingerprints.

## Client trust policy

Client-side trust for SSH sources lives in `~/.pray/trust.toml` (override with `PRAY_HOME`). For `pray+ssh` sources, rules may set:

- `allowed_host_keys` — server host key fingerprints
- `allowed_publishers` — SSH user key fingerprints allowed to publish

Package signatures use `signer_fingerprint` from registry metadata when present.

## Publishing

Publish from a developer machine:

```bash
pray publish --server pray+ssh://pray@your-host
```

Artifacts upload through SSH-RPC (`artifact.put`), then metadata is pushed with `sync.push`. Published metadata records both `signer` (human label) and `signer_fingerprint` (canonical signing identity) when the client knows the SSH fingerprint.

## Verification

Package hashes and signatures are verified on the client the same way as HTTP installs. SSH authenticates the connection; package signatures authenticate the content.

## Local smoke test without OpenSSH

```bash
printf '%s' ... | pray serve --stdio --root ./distribution
```

Integration tests spawn `pray serve --stdio` as a child process and speak framed RPC on stdin and stdout.
