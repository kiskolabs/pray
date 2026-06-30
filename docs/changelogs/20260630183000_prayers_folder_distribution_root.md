# `prayers/` distribution root layout

## What changed
- `pray repo init` now scaffolds a `prayers/` folder containing the distribution repository layout
- `pray install` can resolve a git repository that keeps the distribution tree under `prayers/`
- Documentation now calls out `./prayers` as the recommended local distribution root

## Why it matters
- Distribution data can live alongside other repository content without colliding with project files
- Git repositories remain an easy distribution source even when the distribution root is nested
- The default folder name matches the existing `pray serve --root ./prayers` workflow

## Validation
- `cargo test -p pray --test init_commands`
- `cargo test -p pray --test install_distribution_point install_can_resolve_packages_from_a_git_distribution_repo`
- `cargo test`
- `cargo clippy --workspace -- -D warnings`
- `cargo fmt --all --check`
