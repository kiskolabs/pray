import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { latestConstraintForPackage, versionSatisfies } from "./constraint.js";

describe("constraint", () => {
  it("matches ruby pessimistic constraints", () => {
    assert.equal(versionSatisfies("1.4.3", "~> 1.4"), true);
    assert.equal(versionSatisfies("1.5.0", "~> 1.4"), false);
  });

  it("derives latest constraints by operator style", () => {
    assert.equal(latestConstraintForPackage("~> 1.0", "2.0.0"), "~> 2.0");
    assert.equal(latestConstraintForPackage("1.0.0", "2.0.0"), "=2.0.0");
    assert.equal(latestConstraintForPackage("^1.0", "2.1.0"), "^2.1");
    assert.equal(latestConstraintForPackage("*", "9.0.0"), "*");
  });
});
