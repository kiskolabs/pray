import assert from "node:assert/strict";
import { join, resolve } from "node:path";
import { describe, it } from "node:test";
import { PrayError } from "../errors.js";
import {
  rejectAbsoluteArtifactPath,
  resolveDistributionPath,
  validatePackageName,
  validatePathSegment,
} from "./path-safety.js";

describe("federation path safety", () => {
  it("contains peer paths under the distribution root", () => {
    const root = resolve("tmp", "distribution");
    assert.equal(
      resolveDistributionPath(root, "artifacts/sample.praypkg"),
      join(root, "artifacts/sample.praypkg"),
    );
    for (const path of ["../escape", "/absolute", "C:\\escape"]) {
      assert.throws(
        () => resolveDistributionPath(root, path),
        (error: unknown) => error instanceof PrayError,
      );
    }
  });

  it("rejects unsafe package names and version segments", () => {
    assert.equal(validatePackageName("sample/base"), "sample/base");
    assert.throws(() => validatePackageName("../../outside"));
    assert.equal(validatePathSegment("1.2.3", "version"), "1.2.3");
    assert.throws(() => validatePathSegment("../1.2.3", "version"));
  });

  it("rejects absolute remote artifact URLs", () => {
    assert.throws(() =>
      rejectAbsoluteArtifactPath("https://evil.example/pkg.praypkg"),
    );
    assert.throws(() => rejectAbsoluteArtifactPath("file:///etc/passwd"));
  });
});
