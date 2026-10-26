import assert from "node:assert/strict";
import { describe, it } from "node:test";
import type { PackageSpec } from "./types.js";
import {
  recordedPackageVersion,
  requireReleaseVersion,
  satisfyPackageConstraint,
} from "./version.js";

function spec(version: string): PackageSpec {
  return {
    name: "project",
    version,
    authors: [],
    maintainers: [],
    files: [],
    exports: new Map(),
    skills: new Map(),
    templates: new Map(),
    adapters: new Map(),
    targets: [],
    dependencies: [],
    metadata: new Map(),
  };
}

describe("package spec version", () => {
  it("records local for an omitted version and matches star", () => {
    const packageSpec = spec("");
    assert.equal(recordedPackageVersion(packageSpec), "local");
    satisfyPackageConstraint(packageSpec, "*");
    assert.throws(
      () => satisfyPackageConstraint(packageSpec, "~> 1.0"),
      /has no version/,
    );
    assert.throws(() => requireReleaseVersion(packageSpec), /needs a version/);
  });
});
