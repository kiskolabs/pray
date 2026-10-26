import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { PrayError } from "./errors.js";
import {
  annotateMissingGitCatalog,
  resolutionMayBenefitFromGitSourceRefresh,
} from "./resolve/git-refresh.js";

describe("git catalog refresh matcher", () => {
  it("treats a missing catalog file as a git refresh candidate", () => {
    const error = PrayError.resolution(
      "package sample/extra not found in distribution. Missing v1/packages/sample/extra.json. Check the package name.",
    );
    assert.equal(resolutionMayBenefitFromGitSourceRefresh(error), true);
  });

  it("rewrites a git catalog miss to name the revision and pray update", () => {
    const error = PrayError.resolution(
      "package sample/extra not found in distribution. Check the package name, version constraint `~> 1.0`, and that the source publishes registry metadata.",
    );
    const annotated = annotateMissingGitCatalog(
      error,
      "sample/extra",
      "dist",
      "abc123",
    );
    assert.ok(annotated instanceof PrayError);
    assert.match(annotated.message, /abc123/);
    assert.match(annotated.message, /pray update/);
    assert.equal(annotated.message.includes("check the package name"), false);
  });
});
