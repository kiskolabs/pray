import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { PrayError } from "../../errors.js";
import { parseCleanArguments } from "./clean.js";

describe("clean command arguments", () => {
  it("accepts only the unused flag", () => {
    assert.deepEqual(parseCleanArguments([]), { unused: false });
    assert.deepEqual(parseCleanArguments(["--unused"]), { unused: true });
    assert.throws(
      () => parseCleanArguments(["--other"]),
      (error: unknown) =>
        error instanceof PrayError &&
        error.message.includes("unknown clean flag"),
    );
    assert.throws(
      () => parseCleanArguments(["unused"]),
      (error: unknown) =>
        error instanceof PrayError &&
        error.message.includes("unexpected clean argument"),
    );
  });
});
