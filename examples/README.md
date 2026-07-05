# Examples

This folder contains small, self-contained Prayfile examples that show different ways to compose and customize inference input.

## Included projects

- `simple-project/` — one package, one target, default rendering, and a project note under `.agents/`
- `team-workflow/` — multiple packages, multiple targets, grouped dependencies, and target-specific local input
- `customized-render/` — custom source wiring, optional local input, and an explicit render policy

Each example includes a `Prayfile`, the package definitions it depends on, and a small set of exported fragments that the renderer can assemble.
