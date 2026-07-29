## Participants

- amkisko

## Decisions

- Report position_drift once per target instead of once per marker.
- Include uniform line shift, first marker lock versus file lines, and path:line cause when unmarked preamble differs from fresh composition.
- Prefer attributing cause to a compose local source line when that line is present in resolved local files.

## Effects

- Rust verify/drift/install warnings group marker shifts and cite cause paths.
- npmjs and rubygems verify ports match the grouped position_drift message and cause attribution.
- SPEC.md position_drift row documents the grouped report contract.
- Unit and install_drift coverage added for grouping and local cause attribution.
- Shipped in 1.8.1.

## Next

- Consider multi-line install warning layout if single-line grouped messages become hard to scan.

## Source

- Conversation about AGENTS.md position_drift noise after unmarked preamble edit.
