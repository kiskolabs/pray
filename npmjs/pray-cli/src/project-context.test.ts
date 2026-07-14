import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, beforeEach, describe, it } from "node:test";
import {
  ENV_ENVIRONMENT,
  ENV_MANIFEST_PATH,
  ENV_PROJECT_PATH,
  projectInvocationContextFromOptions,
} from "./project-context/index.js";
import { parseDotenvText } from "./project-context/dotenv.js";

describe("project context", () => {
  const previousEnvironment: Record<string, string | undefined> = {};
  let temporaryDirectory = "";

  beforeEach(() => {
    for (const key of [ENV_PROJECT_PATH, ENV_MANIFEST_PATH, ENV_ENVIRONMENT]) {
      previousEnvironment[key] = process.env[key];
      delete process.env[key];
    }
    temporaryDirectory = mkdtempSync(join(tmpdir(), "pray-context-test-"));
  });

  afterEach(() => {
    for (const [key, value] of Object.entries(previousEnvironment)) {
      if (value === undefined) {
        delete process.env[key];
      } else {
        process.env[key] = value;
      }
    }
    rmSync(temporaryDirectory, { recursive: true, force: true });
  });

  it("parses common dotenv forms for PRAY_* variables only", () => {
    const variables = parseDotenvText(`
# comment
export PRAY_ENV=development
PRAY_PATH="/tmp/project"
PRAY_FILE_PATH='configs/Prayfile'
OTHER=value
`);
    assert.equal(variables.get("PRAY_ENV"), "development");
    assert.equal(variables.get("PRAY_PATH"), "/tmp/project");
    assert.equal(variables.get("PRAY_FILE_PATH"), "configs/Prayfile");
    assert.equal(variables.has("OTHER"), false);
  });

  it("prefers cli options over process environment and dotenv", () => {
    const projectRoot = join(temporaryDirectory, "project");
    mkdirSync(projectRoot, { recursive: true });
    writeFileSync(join(projectRoot, "Prayfile"), 'prayfile "1"\n');
    writeFileSync(
      join(temporaryDirectory, ".env"),
      `PRAY_PATH=ignored-from-dotenv\nPRAY_ENV=ignored-from-dotenv\n`,
    );
    process.env[ENV_PROJECT_PATH] = "ignored-from-process";
    process.env[ENV_ENVIRONMENT] = "ignored-from-process";

    const context = projectInvocationContextFromOptions({
      projectRoot,
      environment: "development",
    });

    assert.equal(context.environment, "development");
    assert.match(context.manifestPath, /project\/Prayfile$/);
    assert.match(context.projectRoot, /project$/);
  });
});
