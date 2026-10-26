import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { parseManifest } from "./index.js";

describe("manifest parser indexes", () => {
  it("merges repeated packages without changing declaration order", () => {
    const manifest = parseManifest(`
prayfile "1"
compose "AGENTS.md" do
  pray "sample/base", "*", export: "first"
  pray "sample/other", "*"
  pray "sample/base", "~> 1.0", export: "second"
end
`);

    assert.deepEqual(
      manifest.packages.map((packageEntry) => packageEntry.name),
      ["sample/base", "sample/other"],
    );
    assert.equal(manifest.packages[0]?.constraint, "~> 1.0");
    assert.deepEqual(manifest.packages[0]?.exports, ["first", "second"]);
    assert.deepEqual(manifest.targets[0]?.entries, [
      { kind: "package", name: "sample/base" },
      { kind: "package", name: "sample/other" },
    ]);
  });
});
