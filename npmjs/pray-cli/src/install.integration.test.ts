import assert from "node:assert/strict";
import { cpSync, mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { describe, it } from "node:test";
import { materializeProject } from "./cli/materialize.js";
import { readLockfile } from "./lockfile/index.js";
import { defaultLockfilePath } from "./lockfile/paths.js";
import { resolveProject } from "./resolve/project.js";
import { verifyProject } from "./verify/project.js";

describe("install integration", () => {
  it("installs and verifies simple-project fixture", async () => {
    const fixtureRoot = resolve(
      import.meta.dirname,
      "../../../examples/simple-project",
    );
    const workspace = mkdtempSync(join(tmpdir(), "pray-cli-install-"));
    cpSync(fixtureRoot, workspace, { recursive: true });

    const manifestPath = join(workspace, "Prayfile");
    process.chdir(workspace);
    await materializeProject({ manifestPath });

    const project = await resolveProject(manifestPath);
    const lockfile = readLockfile(defaultLockfilePath(project.projectRoot));
    verifyProject(project, lockfile, true);

    const rendered = readFileSync(join(workspace, "INSTRUCTIONS.md"), "utf8");
    assert.match(rendered, /<!-- pray:/);
    assert.match(rendered, /\.agents\/project\.md/);
  });
});
