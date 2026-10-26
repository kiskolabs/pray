import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { PrayError } from "../errors.js";
import { validateArchiveMemberPath } from "./path-safety.js";

describe("archive member paths", () => {
  it("collapses current-directory segments", () => {
    assert.equal(
      validateArchiveMemberPath("exports/./guidance.md"),
      "exports/guidance.md",
    );
    assert.equal(validateArchiveMemberPath("./README.md"), "README.md");
  });

  it("rejects parent escapes", () => {
    assert.throws(
      () => validateArchiveMemberPath("../escape.md"),
      (error: unknown) =>
        error instanceof PrayError && error.message.includes("escapes"),
    );
  });
});
