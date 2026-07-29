import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { PrayError } from "../../errors.js";
import { parseUpdateArguments, runUpdateCommand } from "./update.js";

describe("update flags", () => {
  it("parses package and flags", () => {
    const flags = parseUpdateArguments(["demo", "--latest", "--json"]);
    assert.equal(flags.packageName, "demo");
    assert.equal(flags.latest, true);
    assert.equal(flags.json, true);
  });

  it("rejects conflicting major and latest", async () => {
    await assert.rejects(
      () => runUpdateCommand(["demo", "--major", "--latest"]),
      (error: unknown) =>
        error instanceof PrayError &&
        error.message.includes("either --major or --latest"),
    );
  });

  it("rejects major without package", async () => {
    await assert.rejects(
      () => runUpdateCommand(["--major"]),
      (error: unknown) =>
        error instanceof PrayError &&
        error.message.includes("require a package name"),
    );
  });

  it("rejects dry-run with json", async () => {
    await assert.rejects(
      () => runUpdateCommand(["--dry-run", "--json"]),
      (error: unknown) =>
        error instanceof PrayError &&
        error.message.includes("--json is not supported with --dry-run"),
    );
  });
});
