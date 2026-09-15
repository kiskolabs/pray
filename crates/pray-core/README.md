# pray-core

Core library for [Prayfile](https://pray.kisko.dev): parse manifests and package specs, resolve dependencies, write lockfiles, render managed targets, verify drift, and build distribution artifacts.

This is the shared engine used by the `pray` CLI (`pray-cli` on crates.io).

## Install

```toml
[dependencies]
pray-core = "1.5"
```

The default `auth` feature enables local registry authentication storage (SQLite). For a slim build:

```toml
pray-core = { version = "1.5", default-features = false }
```

## Embedder API

Import `pray_core::embed` (RFC 0109). That module is the supported library surface for parse, resolve, lock, render, and verify. `inspect_locked_destinations` checks dest files against `Prayfile.lock` without resolving packages (RFC 0106). Other public modules remain for the in-tree CLI.

```rust
use pray_core::embed::{parse_lockfile, parse_manifest};

let manifest = parse_manifest("prayfile \"1\"\n").expect("manifest");
assert_eq!(manifest.prayfile_version, "1");
let _ = parse_lockfile;
```

## Links

- Spec: [rfcs/](https://github.com/kiskolabs/pray/blob/main/rfcs/README.md)
- CLI: [pray-cli](https://crates.io/crates/pray-cli)
- Homepage: [pray.kisko.dev](https://pray.kisko.dev)
