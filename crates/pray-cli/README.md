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

Shell completions:

```sh
pray completion bash > ~/.local/share/bash-completion/completions/pray
pray completion zsh > ~/.zsh/completions/_pray
pray completion fish > ~/.config/fish/completions/pray.fish
```

Man page (from a checkout): `man ./docs/man/pray.1`. Exit codes: `docs/cli-exit-codes.md`.

## Related crates

- [pray-core](https://crates.io/crates/pray-core) — parsing, resolution, lockfiles, rendering
- [pray-transport](https://crates.io/crates/pray-transport) — distribution-point transports

## Links

- Spec: [rfcs/](https://github.com/kiskolabs/pray/blob/main/rfcs/README.md)
- Homepage: [pray.kisko.dev](https://pray.kisko.dev)
- Repository: [kiskolabs/pray](https://github.com/kiskolabs/pray)
- crates.io: [pray-cli](https://crates.io/crates/pray-cli)
- npm: [pray-cli](https://www.npmjs.com/package/pray-cli)
- RubyGems: [pray-cli](https://rubygems.org/gems/pray-cli)
