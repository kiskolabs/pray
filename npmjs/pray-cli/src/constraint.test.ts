import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { versionSatisfies } from "./constraint.js";

describe("constraint", () => {
  it("matches ruby pessimistic constraints", () => {
    assert.equal(versionSatisfies("1.4.3", "~> 1.4"), true);
    assert.equal(versionSatisfies("1.5.0", "~> 1.4"), false);
  });
});
