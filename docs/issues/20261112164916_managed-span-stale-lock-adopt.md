# Managed span stale lock adopt

## Decisions

Conflict on write compared only the on-disk managed body to the previous lock `ideal_checksum`. When the destination already matched the fresh render ideal and the lock was stale, install failed with `was edited`, and `install --locked` told the operator to rerun install. That loop is not fixed by a general `--force` flag.

Adopt when the on-disk active span body checksum equals the fresh render `ideal_checksum` for that span id. Fail closed when the body equals neither the lock ideal nor the fresh ideal. Do not add `--force` or `--accept-identical-local`. Recovery for a true edit remains move the destination aside and rerun `pray install`, or restore the locked bytes.

Normative home is RFC 0121 (amends RFC 0030 section 45). RFC 0033 adopt-if-identical for provisioned leaves is prior art only; marker dialects stay out of RFC 0033 scope.

`silenced` on `[[managed_span]]` remains written and unread on conflict, verify, and drift. Removal is deferred.

## Effects

Done: RFC 0121 claimed; conflict gates in Rust, Ruby, and TypeScript adopt when disk equals fresh ideal; recovery text names move-aside. Observed: cargo test -p pray-cli --test render_check (4 passed); cargo test -p pray-core --test rfc_ids (5 passed); cargo test -p pray-core (exit 0); cargo test -p pray-cli --test install_validation (14 passed); bundle exec rspec spec/pray/render_reconciliation_spec.rb (8 examples, 0 failures); node --test render-check adopt/refuse cases (2 passed).

Claim: conflict compared dest only to lock `ideal_checksum`. Outcome: supported before the fix; outdated after RFC 0121 adopt. Source: crates/pray-core/src/render_conflict.rs; rubygems/pray-cli/lib/pray/render_reconcile.rb; npmjs/pray-core/src/render/conflict.ts.

Claim: fresh render ideal was unused in the conflict gate. Outcome: supported. Source: same modules; rendered targets supplied only the active span id set.

Claim: RFC 0033 rejects `--force` and `dest.bak` for provisioned destinations. Outcome: supported. Source: rfcs/0033-provisioned-destination-safety.md rationale.

Claim: RFC 0033 already specified adopt-if-identical for managed marker spans. Outcome: unsupported. Source: rfcs/0033-provisioned-destination-safety.md summary (marker dialects out of scope); rfcs/0030-render-markers-ownership.md section 45 (lock ideal only).

Claim: `install --locked` and plain install can form a recovery loop when lock ideals are stale and dest already matches fresh. Outcome: supported. Source: crates/pray-cli/src/lockfile_ops.rs; crates/pray-core/src/render_conflict.rs; crates/pray-cli/tests/install_validation.rs.

Claim: `silenced` is unread outside lock parse and serialize. Outcome: supported. Source: crates/pray-core/src/lockfile.rs; crates/pray-core/src/render_span.rs (always false on write).

Claim: drift and plan diagnose "stale lock, content agrees" as a distinct finding. Outcome: unsupported before this work. Source: crates/pray-core/src/verify/mod.rs (`custom_implementation`); crates/pray-cli/src/commands_inspect.rs.

## Next

Improve drift or plan wording for the stale-lock case in a later pass. Decide whether to use or remove `silenced`. Advance RFC 0121 toward Stable after the adopt behavior has shipped long enough.

## Source

rfcs/0030-render-markers-ownership.md
rfcs/0033-provisioned-destination-safety.md
rfcs/0121-managed-span-adopt-if-identical.md
crates/pray-core/src/render_conflict.rs
docs/issues/20260916180300_install-update-verify-local-compose.md
