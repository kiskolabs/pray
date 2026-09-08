import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  assertPathUpstreamRefreshSupported,
  ensureLockedUpstreamMatches,
} from "./upstream.js";

describe("package upstream refresh", () => {
  it("refuses an update before writing an unsupported path refresh", () => {
    assert.throws(
      () =>
        assertPathUpstreamRefreshSupported([
          {
            declaration: { name: "fork/base", path: "packages/base" },
            spec: { upstream: { name: "sample/base", constraint: "~> 1.4" } },
          },
        ]),
      /cannot refresh upstream package fork\/base/,
    );
  });

  it("rejects content that no longer matches the locked upstream", () => {
    assert.throws(
      () =>
        ensureLockedUpstreamMatches(
          {
            name: "sample/base",
            version: "1.4.3",
            source: "sample",
            tree_hash: "sha256:old",
            artifact_hash: "sha256:artifact",
          },
          {
            name: "sample/base",
            version: "1.4.3",
            source: "sample",
            tree_hash: "sha256:changed",
            artifact_hash: "sha256:artifact",
          },
        ),
      /locked upstream tree hash mismatch/,
    );
  });
});
