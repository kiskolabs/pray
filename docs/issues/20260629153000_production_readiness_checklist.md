# Production readiness checklist

This checklist is prioritized for the current `pray` codebase and its documented goals.

## Must-have before beta

- [ ] Complete one fully realistic end-to-end happy path:
  - fresh machine
  - login
  - confess
  - publish
  - install/use from another machine
  - sync changes
- [ ] Prove `publish` works over the network in the supported distribution flow.
- [ ] Prove `login`, `confess`, and `sync` survive common network failure and recovery cases.
- [ ] Cover at least one interruption case during upload, not only before or after the request.
- [ ] Verify docs and CLI help text match the actual supported network behavior.
- [ ] Keep the shipped command set stable and understandable for first-time users.
- [ ] Ensure diagnostics and exit behavior are clear enough for friend-led beta support.

## Must-have before production

- [ ] Cover a true cross-machine workflow with two users or two environments consuming the same published result.
- [ ] Test stale, partial, and corrupted local state recovery for all persisted formats that matter.
- [ ] Add failure/retry coverage for partial network operations, auth expiry, and bad server responses.
- [ ] Validate upgrade and migration behavior for any user-facing stored state.
- [ ] Confirm trust, publish, and sync behavior under real network conditions, including restarts.
- [ ] Validate supported platforms with the same flows that real users will run.
- [ ] Tighten support boundaries so the app can be operated without manual repair in normal cases.
- [ ] Keep the codebase within the file-size rule and split large logic into semantic modules/helpers.

## Nice-to-have after launch

- [ ] Expand federation and distribution options.
- [ ] Add more failure-injection and chaos-style tests.
- [ ] Improve diagnostics, UX polish, and troubleshooting guidance.
- [ ] Add performance and scale checks.
- [ ] Broaden ecosystem integrations once the core flows are stable.
- [ ] Add deeper observability if it helps support and debugging.

## Current next slice

The highest-value next slice is a full cross-machine test that proves one person can publish and another can consume the result from the network, with one injected failure and recovery step.
