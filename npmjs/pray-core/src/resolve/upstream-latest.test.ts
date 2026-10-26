import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { latestSpecUpstreamConstraint } from "./upstream-latest.js";

describe("path upstream latest constraints", () => {
  it("uses a spaced equals pin", () => {
    assert.equal(latestSpecUpstreamConstraint("= 1.4.3", "1.4.4"), "= 1.4.4");
  });

  it("follows the latest operator family for a pessimistic pin", () => {
    assert.equal(latestSpecUpstreamConstraint("~> 1.4", "2.0.0"), "~> 2.0");
  });
});
