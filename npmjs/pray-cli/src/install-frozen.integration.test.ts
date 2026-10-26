import assert from "node:assert/strict";
import {
  existsSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, it } from "node:test";
import { materializeProject } from "pray-core/materialize.js";
import { writePackage } from "pray-core/render-test-package.js";
import { runCli } from "./cli/main.js";

describe("locked and frozen installation", () => {
  const originalDirectory = process.cwd();
  let root: string;

  afterEach(() => {
    process.chdir(originalDirectory);
    if (root) rmSync(root, { recursive: true, force: true });
  });

  function setup(): void {
    root = mkdtempSync(join(tmpdir(), "pray-install-frozen-"));
    process.chdir(root);
    writePackage(
      root,
      "rules",
      "sample/rules",
      "rules",
      "file",
      "exports/rules.md",
      "Keep it small.\n",
    );
    writeFileSync(
      join(root, "Prayfile"),
      `prayfile "1"
compose "AGENTS.md" do
  pray "sample/rules", "~> 1.0", path: "packages/rules"
end
pray "sample/rules", "~> 1.0", path: "packages/rules", file: "rules.txt"
`,
    );
  }

  function snapshot(paths: string[]) {
    return paths.map((path) => {
      const absolute = join(root, path);
      if (!existsSync(absolute)) return undefined;
      const stats = statSync(absolute);
      return {
        bytes: readFileSync(absolute),
        inode: stats.ino,
        modified: stats.mtimeMs,
      };
    });
  }

  const outputs = ["AGENTS.md", "rules.txt", "Prayfile.lock"];

  for (const flag of ["--locked", "--frozen"]) {
    it(`${flag} rejects a missing lock before creating outputs`, async () => {
      setup();
      assert.equal(await runCli(["install", flag]), 3);
      assert.deepEqual(snapshot(outputs), [undefined, undefined, undefined]);
    });

    it(`${flag} rejects a changed lock without modifying outputs`, async () => {
      setup();
      await materializeProject();
      writeFileSync(
        join(root, "Prayfile"),
        readFileSync(join(root, "Prayfile"), "utf8").replace(
          'compose "AGENTS.md"',
          'compose "AGENTS.md", header: false',
        ),
      );
      const before = snapshot(outputs);
      assert.equal(await runCli(["install", flag]), 3);
      assert.deepEqual(snapshot(outputs), before);
    });
  }

  for (const path of ["AGENTS.md", "rules.txt"]) {
    it(`frozen rejects missing ${path} without repairing it`, async () => {
      setup();
      await materializeProject();
      rmSync(join(root, path));
      const before = snapshot(outputs);
      assert.equal(await runCli(["install", "--frozen"]), 6);
      assert.deepEqual(snapshot(outputs), before);
    });
  }

  it("frozen rejects edited compose content without repairing it", async () => {
    setup();
    await materializeProject();
    const path = join(root, "AGENTS.md");
    writeFileSync(
      path,
      readFileSync(path, "utf8").replace("Keep it small.", "Keep my edits."),
    );
    const before = snapshot(outputs);
    assert.equal(await runCli(["install", "--frozen"]), 6);
    assert.deepEqual(snapshot(outputs), before);
  });

  it("frozen rejects changed sources before writing either destination", async () => {
    setup();
    await materializeProject();
    writeFileSync(
      join(root, "packages/rules/exports/rules.md"),
      "Updated rules.\n",
    );
    const before = snapshot(outputs);
    assert.equal(await runCli(["install", "--frozen"]), 3);
    assert.deepEqual(snapshot(outputs), before);
  });

  it("frozen rejects removed provisioned destinations before pruning", async () => {
    setup();
    await materializeProject();
    writeFileSync(join(root, "Prayfile"), 'prayfile "1"\n');
    const before = snapshot(outputs);
    assert.equal(await runCli(["install", "--frozen"]), 3);
    assert.deepEqual(snapshot(outputs), before);
  });

  for (const entry of ["cli", "library"]) {
    it(`successful frozen ${entry} preserves unmanaged text and lock formatting without writes`, async () => {
      setup();
      await materializeProject();
      for (const [path, addition] of [
        ["AGENTS.md", "\nMy local notes.\n"],
        ["Prayfile.lock", "\n# Preserve my lock comment.\n"],
      ]) {
        writeFileSync(
          join(root, path!),
          `${readFileSync(join(root, path!), "utf8")}${addition}`,
        );
      }
      rmSync(join(root, ".pray/write-state"), { recursive: true });
      const before = snapshot(outputs);
      if (entry === "cli")
        assert.equal(await runCli(["install", "--frozen"]), 0);
      else await materializeProject({ frozen: true });
      assert.deepEqual(snapshot(outputs), before);
      assert.equal(existsSync(join(root, ".pray/write-state")), false);
    });
  }
});
