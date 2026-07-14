import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, beforeEach, describe, it } from "node:test";
import { initializeInvocation } from "./invocation.js";
import {
  ENV_ENVIRONMENT,
  ENV_PROJECT_PATH,
} from "../project-context/index.js";
import { setActiveInvocationContext } from "../project-context/runtime.js";

describe("cli invocation", () => {
  const previousEnvironment: Record<string, string | undefined> = {};
  let temporaryDirectory = "";

  beforeEach(() => {
    for (const key of [ENV_PROJECT_PATH, ENV_ENVIRONMENT]) {
      previousEnvironment[key] = process.env[key];
      delete process.env[key];
    }
    temporaryDirectory = mkdtempSync(join(tmpdir(), "pray-invocation-test-"));
    setActiveInvocationContext(undefined);
  });

  afterEach(() => {
    for (const [key, value] of Object.entries(previousEnvironment)) {
      if (value === undefined) {
        delete process.env[key];
      } else {
        process.env[key] = value;
      }
    }
    setActiveInvocationContext(undefined);
    rmSync(temporaryDirectory, { recursive: true, force: true });
  });

  it("strips global flags before the subcommand", () => {
    const projectRoot = join(temporaryDirectory, "project");
    mkdirSync(projectRoot, { recursive: true });
    writeFileSync(join(projectRoot, "Prayfile"), 'prayfile "1"\n');
    process.env[ENV_PROJECT_PATH] = "ignored";
    process.env[ENV_ENVIRONMENT] = "ignored";

    const remaining = initializeInvocation([
      "--path",
      projectRoot,
      "--env",
      "development",
      "plan",
    ]);

    assert.deepEqual(remaining, ["plan"]);
  });
});
