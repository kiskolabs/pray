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
import { afterEach, describe, it } from "node:test";
import { PrayError } from "../errors.js";
import { materializeProject } from "../materialize.js";
import { writePackage } from "../render-test-package.js";
import { resolveProject } from "../resolve/project.js";
import { plannedProvisionedFiles } from "./provisioned.js";

describe("provisioned destination ownership", () => {
  const originalDirectory = process.cwd();
  let root: string;

  afterEach(() => {
    process.chdir(originalDirectory);
    if (root) rmSync(root, { recursive: true, force: true });
  });

  function setup(): void {
    root = mkdtempSync(join(tmpdir(), "pray-provisioned-collision-"));
    process.chdir(root);
    for (const name of ["first", "second"]) {
      writePackage(
        root,
        name,
        `sample/${name}`,
        "rules",
        "file",
        "exports/rules.md",
        "Shared rules.\n",
      );
    }
  }

  async function rejectCollision(
    path: string,
    owners: string[],
  ): Promise<void> {
    await assert.rejects(
      () => materializeProject(),
      (error: unknown) =>
        error instanceof PrayError &&
        error.kind === "render" &&
        error.message.includes(path) &&
        owners.every((owner) => error.message.includes(owner)),
    );
    assert.equal(existsSync(join(root, path)), false);
    assert.equal(existsSync(join(root, "Prayfile.lock")), false);
  }

  it("rejects two packages targeting one file even when their bytes match", async () => {
    setup();
    writeFileSync(
      join(root, "Prayfile"),
      `prayfile "1"
pray "sample/first", path: "packages/first", file: "rules.txt"
pray "sample/second", path: "packages/second", file: "rules.txt"
`,
    );
    await rejectCollision("rules.txt", [
      "sample/first::rules",
      "sample/second::rules",
    ]);
  });

  it("rejects two exports in one package targeting one tree leaf", async () => {
    setup();
    writeFileSync(
      join(root, "packages/first/first.prayspec"),
      `Package::Specification.new do |spec|
  spec.name = "sample/first"
  spec.version = "1.0.0"
  spec.files = ["exports/rules.md"]
  spec.exports = {
    "primary" => { type: "folder", path: "exports" },
    "secondary" => { type: "folder", path: "exports" }
  }
end
`,
    );
    writeFileSync(
      join(root, "Prayfile"),
      `prayfile "1"
tree "rules" do
  pray "sample/first", path: "packages/first", exports: ["primary", "secondary"]
end
`,
    );
    await rejectCollision("rules/exports/rules.md", [
      "sample/first::primary",
      "sample/first::secondary",
    ]);
  });

  it("keeps repeated bindings to the same package export usable", async () => {
    setup();
    writeFileSync(
      join(root, "Prayfile"),
      `prayfile "1"
target :first do
  skills "rules"
end
target :second do
  skills "rules"
end
pray "sample/first", path: "packages/first"
`,
    );
    const project = await resolveProject(join(root, "Prayfile"));
    assert.equal(plannedProvisionedFiles(project).length, 1);
    await materializeProject();
    assert.equal(
      readFileSync(join(root, "rules/rules/rules.md"), "utf8"),
      "Shared rules.\n",
    );
  });
});
