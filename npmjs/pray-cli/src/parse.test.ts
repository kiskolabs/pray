import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  lockfileHash,
  manifestHash,
  parseLockfile,
  parseManifest,
  serializeLockfile,
} from "./parse.js";

describe("pray-cli parse entry", () => {
  it("re-exports parse APIs from pray-core/parse", () => {
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

    assert.equal(manifest.prayfileVersion, "1");
    assert.equal(lockfile.prayfile_lock, "1");
    assert.equal(manifestHash(manifest).startsWith("sha256:"), true);
    assert.equal(lockfileHash(lockfile).startsWith("sha256:"), true);
    assert.equal(
      serializeLockfile(lockfile).includes('manifest_hash = "sha256:abc"'),
      true,
    );
  });
});
