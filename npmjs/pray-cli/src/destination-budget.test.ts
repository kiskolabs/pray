import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { it } from "node:test";
import { provisionedDestinationStatuses } from "./render/dest.js";
import { resolveProject } from "./resolve/project.js";

it("bounds collision reports and counts omitted paths", async () => {
  const root = mkdtempSync(join(tmpdir(), "pray-destination-budget-"));
  try {
    mkdirSync(join(root, "package/files"), { recursive: true });
    mkdirSync(join(root, "out/files"), { recursive: true });
    const files = Array.from(
      { length: 105 },
      (_, index) => `files/${index.toString().padStart(3, "0")}.txt`,
    );
    writeFileSync(
      join(root, "Prayfile"),
      'prayfile "1"\ntree "out" do\n pray "sample/files", path: "package"\nend\n',
    );
    writeFileSync(
      join(root, "package/files.prayspec"),
      `Package::Specification.new do |spec|\n spec.name = "sample/files"\n spec.version = "1.0.0"\n spec.files = ${JSON.stringify(files)}\n spec.exports = { "files" => { type: "folder", path: "files" } }\nend\n`,
    );
    for (const file of files) {
      writeFileSync(join(root, "package", file), "package");
      writeFileSync(join(root, "out", file), "operator");
    }
    const project = await resolveProject(join(root, "Prayfile"));
    assert.throws(
      () => provisionedDestinationStatuses(project),
      (error: unknown) => {
        const message = String(error);
        assert.ok(
          message.includes("5 additional destination conflicts"),
          message,
        );
        assert.ok(Buffer.byteLength(message) < 65536);
        return true;
      },
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
