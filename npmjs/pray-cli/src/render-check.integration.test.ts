import assert from "node:assert/strict";
import {
  existsSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { renderProject } from "pray-core/render/project.js";
import { writePackage } from "pray-core/render-test-package.js";
import { resolveProject } from "pray-core/resolve/project.js";
import { runCli } from "./cli/main.js";

async function writeProject(root: string): Promise<string> {
  writePackage(
    root,
    "rules",
    "sample/rules",
    "rules",
    "fragment",
    "exports/rules.md",
    "Keep the check read-only.\n",
  );
  writeFileSync(
    join(root, "Prayfile"),
    `
prayfile "1"
compose "AGENTS.md" do
  pray "sample/rules", "~> 1.0", path: "packages/rules"
end
`,
  );
  const project = await resolveProject(join(root, "Prayfile"));
  const content = renderProject(project)[0]?.content;
  assert.ok(content);
  writeFileSync(join(root, "AGENTS.md"), content);
  return content;
}

describe("render --check", () => {
  it("accepts a current destination without writing project state", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-render-check-current-"));
    try {
      const content = await writeProject(root);

      assert.equal(await runCli(["--path", root, "render", "--check"]), 0);
      assert.equal(readFileSync(join(root, "AGENTS.md"), "utf8"), content);
      assert.equal(existsSync(join(root, ".pray")), false);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it("rejects a stale destination without changing it", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-render-check-stale-"));
    try {
      await writeProject(root);
      writeFileSync(join(root, "AGENTS.md"), "stale destination\n");

      assert.equal(await runCli(["--path", root, "render", "--check"]), 5);
      assert.equal(
        readFileSync(join(root, "AGENTS.md"), "utf8"),
        "stale destination\n",
      );
      assert.equal(existsSync(join(root, ".pray")), false);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});
