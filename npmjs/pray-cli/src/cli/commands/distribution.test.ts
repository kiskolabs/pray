import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { PrayError } from "../../errors.js";
import { runConfess, runServe, runSync } from "./distribution.js";

function isUsageError(error: unknown): boolean {
  return error instanceof PrayError && error.exitCode() === 2;
}

describe("distribution command arguments", () => {
  it("rejects unknown arguments before IO", async () => {
    await assert.rejects(() => runServe(["--unknown"]), isUsageError);
    await assert.rejects(() => runSync(["--unknown"]), isUsageError);
    await assert.rejects(() => runConfess(["--unknown"]), isUsageError);
  });

  it("rejects missing and malformed option values", async () => {
    await assert.rejects(() => runServe(["--root"]), isUsageError);
    await assert.rejects(() => runServe(["--port", "12x"]), isUsageError);
    await assert.rejects(() => runSync(["--peer"]), isUsageError);
    await assert.rejects(() => runConfess(["--url"]), isUsageError);
  });
});
