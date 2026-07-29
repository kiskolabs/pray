# Deep safety fixes from engineering audit

## Participants

- Andrei Makarov

## Decisions

- Prefer structural safety contracts over soft gates: project-relative path newtype, serve ingress caps, federation visited-peer set, required tree hash on sync, publisher allowlist on package signer, resource quotas, and SPEC exit code 7.
- Reject undeclared required package dependencies until true transitive resolve lands.
- Enforce default render conflict:fail against the previous lockfile so package updates are not false conflicts.
- Reject unimplemented render conflict/mode/churn values at parse time; leave section_markers and line_endings parseable for existing examples.
- Fix CI mutants to examine (-f) parser and integrity surfaces instead of excluding them.

## Effects

- Added ProjectRelativePath and manifest-time validation for compose outputs, tree roots, file destinations, package paths, and local embeds.
- Serve caps concurrent connections, headers, bodies, and socket timeouts; --allow-open-push is parseable and passed through.
- Federation sync uses a permanent visited set and peer cap; blank tree_hash is rejected; signatures verify when present.
- allowed_publishers gates selected package signer fingerprints on registry and SSH installs; consumer SSH identity is no longer used as publisher proof at session open.
- Archive and torrent paths enforce size and entry quotas; HTTP fetch failures map to PrayError::Network (exit 7).
- Multi-output targets render every declared output; max_bytes is enforced before write.
- Mutation smoke Makefile/CI/mutants.toml use examine globs; conflict install test is enabled.

## Next

- Implement work-queue transitive resolve with constraint merge instead of reject-undeclared.
- Unify HTTP and torrent stacks behind one bounded transport owner.
- Implement managed-patch install path (still ignored) and section_markers/line_endings behavior.
- Stream torrent pieces to temp files instead of whole-artifact allocation once quotas are in place.
- Raise coverage floors and make a narrowed mutants baseline blocking.

## Source

- Audit: usr/docs/issues/20260729115833_engineering-audit-post-quality-checks.md
- Branch: feature/deep-safety-fixes
