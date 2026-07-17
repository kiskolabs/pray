import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "node:test";
import {
  inspectProject,
  lockfileHash,
  manifestHash,
  parseLockfile,
  parseManifest,
  readLockfile,
  serializeLockfile,
} from "./index.js";

describe("library API", () => {
  it("exports manifest and lockfile parsers for embedding", () => {
    const manifest = parseManifest(`
prayfile "1"
source "default", "https://agents.example.com"
target :tool_a do
  output "INSTRUCTIONS.md"
end
agent "sample/base", "~> 1.4"
`);

    const lockfile = parseLockfile(`
prayfile_lock = "1"
spec = "0.1"
generated_by = "pray test"
manifest_hash = "sha256:abc"
source = []
package = []
target = []
managed_span = []
`);

    assert.equal(manifest.prayfileVersion, "1");
    assert.equal(lockfile.prayfile_lock, "1");
    assert.equal(manifestHash(manifest).startsWith("sha256:"), true);
    assert.equal(lockfileHash(lockfile).startsWith("sha256:"), true);
    assert.equal(
      serializeLockfile(lockfile).includes('manifest_hash = "sha256:abc"'),
      true,
    );
  });

  it("parses fixture lockfile from text and file with the same result", () => {
    const fixturePath = resolve(
      import.meta.dirname,
      "../../../examples/simple-project/Prayfile.lock",
    );
    const text = readFileSync(fixturePath, "utf8");
    const fromText = parseLockfile(text);
    const fromPath = readLockfile(fixturePath);

    assert.equal(fromText.package[0]?.name, fromPath.package[0]?.name);
    assert.equal(fromText.managed_span.length, fromPath.managed_span.length);
  });

  it("returns verification findings without throwing", () => {
    const manifest = parseManifest(`
prayfile "1"
source "default", "https://agents.example.com"
target :tool_a do
  output "INSTRUCTIONS.md"
end
`);
    const lockfile = parseLockfile(`
prayfile_lock = "1"
spec = "0.1"
generated_by = "pray test"
manifest_hash = "sha256:stale"
source = []
package = []
target = [{ name = "tool_a", outputs = ["INSTRUCTIONS.md"] }]
managed_span = []
`);

    const report = inspectProject(
      {
        manifestPath: "/tmp/Prayfile",
        projectRoot: "/tmp",
        manifest,
        manifestHash: manifestHash(manifest),
        packages: [],
        localFiles: [],
        sourceRevisions: new Map(),
        sourceHostKeys: new Map(),
      },
      lockfile,
    );

    assert.equal(
      report.findings.some((finding) =>
        finding.message.includes("Prayfile changed"),
      ),
      true,
    );
  });
});
