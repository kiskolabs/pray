import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { parseManifest } from "./index.js";
import { manifestToJson } from "./types.js";

// The canonical manifest JSON is a cross-implementation contract: manifest_hash
// in Prayfile.lock is computed from these bytes. A locator the Ruby and Rust
// CLIs emit as null cannot be omitted here, or the same Prayfile hashes
// differently depending on which CLI ran it.
const PRAYFILE = `prayfile "1"

source "amkisko", git: "https://github.com/amkisko/prayers.git"

compose "AGENTS.md" do
  pray "amkisko/working-rules", "~> 2.1"
end
`;

function json(): Record<string, unknown> {
  return manifestToJson(parseManifest(PRAYFILE));
}

describe("manifestToJson", () => {
  it("emits absent package locators as null", () => {
    const packages = json().packages as Record<string, unknown>[];
    const entry = packages[0]!;

    for (const key of [
      "source",
      "path",
      "git",
      "tag",
      "rev",
      "tarball",
      "oci",
    ]) {
      assert.equal(key in entry, true, `expected ${key} to be present`);
      assert.equal(entry[key], null, `expected ${key} to be null`);
    }
  });

  it("emits an absent target max_bytes as null", () => {
    const targets = json().targets as Record<string, unknown>[];
    const target = targets[0]!;

    assert.equal("max_bytes" in target, true);
    assert.equal(target.max_bytes, null);
  });

  it("omits source locators that are absent", () => {
    const sources = json().sources as Record<string, unknown>[];

    assert.deepEqual(Object.keys(sources[0]!), ["name", "kind", "url"]);
  });
});
