import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  lockfileHash,
  manifestHash,
  normalizeVersionConstraint,
  parseLockfile,
  parseManifest,
  parsePackageSpec,
  serializeLockfile,
} from "./parse.js";

describe("parse entry", () => {
  it("parses manifest, lockfile, and package spec text", () => {
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
manifest_hash = "sha256:abc"
source = []
package = []
target = []
managed_span = []
`);
    const spec = parsePackageSpec(`
Package::Specification.new do |spec|
  spec.name = "demo/sample"
  spec.version = "1.0.0"
  spec.files = ["README.md"]
  spec.exports = { "readme" => { type: "file", path: "README.md" } }
end
`);

    assert.equal(manifest.prayfileVersion, "1");
    assert.equal(lockfile.prayfile_lock, "1");
    assert.equal(spec.name, "demo/sample");
    assert.equal(manifestHash(manifest).startsWith("sha256:"), true);
    assert.equal(lockfileHash(lockfile).startsWith("sha256:"), true);
    assert.equal(
      serializeLockfile(lockfile).includes('manifest_hash = "sha256:abc"'),
      true,
    );
    assert.equal(normalizeVersionConstraint("1.2.3"), "=1.2.3");
  });
});
