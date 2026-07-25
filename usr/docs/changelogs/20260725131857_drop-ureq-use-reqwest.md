# Drop ureq in favor of reqwest

## Participants

- amkisko

## Decisions

- Replace pray-cli's ureq calls with the existing reqwest HTTP stack.
- Run the asynchronous reqwest client on a current-thread Tokio runtime for synchronous CLI paths.
- Leave the optional auth sqlite feature-gate out of this change.

## Effects

- Release checks and compromised-key feed retrieval use reqwest.
- Remove ureq from pray-cli dependencies.
- Add coverage for retrieving a compromised-key feed over HTTP.

## Next

- Optional auth sqlite feature-gate remains open.

## Source

- Branch: patch/drop-ureq-use-reqwest
