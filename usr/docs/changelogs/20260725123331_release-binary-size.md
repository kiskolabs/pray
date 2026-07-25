# Release binary size

## Participants

- amkisko

## Decisions

- Add workspace `[profile.release]` with thin LTO, codegen-units = 1, strip, and panic = abort.
- Replace Tokio `full` with the features the CLI and transport actually use (`rt`, `macros`, plus `net`/`io-util` in transport).
- Build reqwest with `default-features = false` and rustls JSON only.
- Keep ureq for sync CLI fetches in this pass; unifying HTTP stacks is deferred.

## Effects

- Release builds strip symbols and apply thin LTO by default.
- Smaller dependency surface for Tokio and reqwest.
- Measured arm64 macOS release size: 13.2 MB before, 8.5 MB after (-36%).

## Next

- Consider consolidating ureq and reqwest onto one HTTP client.

## Source

- Binary size analysis session (timely, scout, status, pray)
