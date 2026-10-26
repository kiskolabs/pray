import assert from "node:assert/strict";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, it } from "node:test";
import { materializeProject } from "pray-core/materialize.js";
import { activeInvocationContext } from "pray-core/project-context/runtime.js";
import { runCli } from "./cli/main.js";

function writePathPackage(root: string, body: string): void {
  const packageRoot = join(root, "packages/shell");
  mkdirSync(join(packageRoot, "exports"), { recursive: true });
  writeFileSync(
    join(packageRoot, "shell.prayspec"),
    `
Package::Specification.new do |spec|
  spec.name = "sample/shell"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = ["exports/zshrc"]
  spec.exports = {
    "zshrc" => { type: "file", path: "exports/zshrc" }
  }
end
`,
  );
  writeFileSync(join(packageRoot, "exports/zshrc"), body);
  writeFileSync(
    join(root, "Prayfile"),
    `
prayfile "1"
pray "sample/shell", "~> 1.0", path: "packages/shell", file: ".zshrc"
`,
  );
}

describe("runCli invocation context", () => {
  const previousWorkingDirectory = process.cwd();

  afterEach(() => {
    process.chdir(previousWorkingDirectory);
  });

  it("installs the current project after runCli in a removed directory", async () => {
    const first = mkdtempSync(join(tmpdir(), "pray-cli-context-first-"));
    const second = mkdtempSync(join(tmpdir(), "pray-cli-context-second-"));
    try {
      writePathPackage(first, "first aliases\n");
      writePathPackage(second, "second aliases\n");
      process.chdir(first);
      assert.equal(await runCli(["install"]), 0);
      rmSync(first, { recursive: true, force: true });
      assert.equal(activeInvocationContext(), undefined);

      process.chdir(second);
      await materializeProject({ manifestPath: join(second, "Prayfile") });
      assert.equal(
        readFileSync(join(second, ".zshrc"), "utf8"),
        "second aliases\n",
      );
    } finally {
      process.chdir(previousWorkingDirectory);
      rmSync(first, { recursive: true, force: true });
      rmSync(second, { recursive: true, force: true });
    }
  });

  it("rejects an unknown install flag before writing", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-cli-unknown-flag-"));
    try {
      writePathPackage(root, "aliases\n");
      process.chdir(root);

      assert.equal(await runCli(["install", "--unknown"]), 2);
      assert.throws(() => readFileSync(join(root, ".zshrc"), "utf8"));
    } finally {
      process.chdir(previousWorkingDirectory);
      rmSync(root, { recursive: true, force: true });
    }
  });
});
