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

## Links

- Spec: [SPEC.md](https://github.com/kiskolabs/pray/blob/main/SPEC.md)
- CLI: [pray-cli](https://crates.io/crates/pray-cli)
- Homepage: [pray.kisko.dev](https://pray.kisko.dev)
