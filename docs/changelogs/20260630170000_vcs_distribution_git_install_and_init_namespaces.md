# VCS-backed distribution sources and init namespaces

## What changed
- `pray install` can resolve packages from git-hosted distribution roots without requiring `pray serve`
- `pray publish` keeps git-backed distribution roots reviewable with commit-and-push behavior, and rejects non-fast-forward pushes so the user can resolve conflicts locally
- Added `pray prayer init` for package repository scaffolding and `pray repo init` for distribution repository scaffolding

## Why it matters
- Git repositories can now act as a simple distribution format for prayers
- Distribution roots can be published, versioned, and shared without an always-on server
- The init split makes consumer projects, package repositories, and distribution repositories easier to set up correctly

## Validation
- `cargo test -p pray --test install_distribution_point install_can_resolve_packages_from_a_git_distribution_repo`
- `cargo test -p pray --test revision`
- `cargo test -p pray --test init_commands`
