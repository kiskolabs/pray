# Host-language lock adapter

## Participants

Andrei Makarov

## Decisions

Claim RFC 0106. Ship inspect_locked_destinations on pray_core::embed. Check dest files against locked managed spans without resolve or render. Leave provisioned files and position drift for a later pass. Do not add Ruby or TypeScript ports in this pass. Do not add a Python package or Mix task until a real in-repo caller exists.

## Effects

crates/pray-core/src/verify/locked_dest.rs holds marker scan and inspect_locked_destinations. verify/mod.rs dropped below the 300 line ceiling by moving marker helpers. cargo test -p pray-core --test locked_dest is the confirming check.

Resource and budget: dest reads use the existing destination size limit. No network. Confirming check: destination_budget tests still pass.

Trace and identification: reads project dest files only. No new identifiers.

Boundary and control: skipped. File reads under project_root.

Product surface: skipped.

Privacy: skipped.

Performance: skipped as unmeasured. Inference: cheaper than verify_project because it does not fetch or re-render. Confirming check: pray-bench destinations example if a later pass needs a number.

Observability: skipped.

Security: dest paths come from the lock. RFC 0106 requires repository-relative paths already recorded by install.

Contract: RFC 0106 Experimental.

Learned systems: skipped.

Python and Elixir: house stack wants them. A fourth resolver is still rejected. Mix and pytest should spawn pray or bind inspect_locked_destinations later. No hex or PyPI tree in this pass.

## Next

PyO3 bind of embed after RFC 0109 field freeze. Mix task when a Phoenix repo asks.

## Source

rfcs/0106-host-language-lock-adapter.md
crates/pray-core/src/verify/locked_dest.rs
crates/pray-core/tests/locked_dest.rs
usr/docs/changelogs/20260915130000_host-language-lock-adapter.md
usr/docs/issues/20260915133000_render-fixtures-and-lock-adapter-ports.md
