import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { removeManifestStatement } from "./edit.js";

describe("removeManifestStatement", () => {
  it("removes a pray declaration written by add", () => {
    const text = `prayfile "1"
pray "sample/base", path: "packages/base"
render mode: :managed
`;
    const updated = removeManifestStatement(text, "sample/base");
    assert.equal(updated.includes("sample/base"), false);
    assert.equal(updated.includes("render mode: :managed"), true);
  });
});
