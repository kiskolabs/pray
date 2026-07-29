# pray-cli

`pray` CLI for [Prayfile](https://pray.kisko.dev): a package manager for the language placed before inference.

Install, lock, render, verify, publish, and serve shared instructions, policies, templates, and related input as versioned packages.

On crates.io the package is `pray-cli` because `pray` is already taken. The installed executable is `pray`.

## Install

```sh
cargo install pray-cli --locked
```

From a git checkout:

```sh
cargo install --path crates/pray-cli --locked
```

## Usage

```sh
pray install
pray verify --strict
pray package
pray publish --root ./prayers
pray serve --root ./prayers --port 7429
```

## Related crates

- [pray-core](https://crates.io/crates/pray-core) — parsing, resolution, lockfiles, rendering
- [pray-transport](https://crates.io/crates/pray-transport) — distribution-point transports

## Links

- Spec: [SPEC.md](https://github.com/kiskolabs/pray/blob/main/SPEC.md)
- Homepage: [pray.kisko.dev](https://pray.kisko.dev)
- Repository: [kiskolabs/pray](https://github.com/kiskolabs/pray)
