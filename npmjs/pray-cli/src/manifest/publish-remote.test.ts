import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { manifestHash, parseManifest } from "./index.js";

describe("publish remotes", () => {
  it("parses path and url remotes", () => {
    const manifest = parseManifest(`
prayfile "1"
source "local", path: "guidance"
publish "prayers", path: "prayers"
publish "public", "https://prayers.example"
compose "AGENTS.md" do
  pray "local/project"
end
`);
    assert.equal(manifest.publishRemotes?.length, 2);
    assert.equal(manifest.publishRemotes?.[0]?.name, "prayers");
    assert.equal(manifest.publishRemotes?.[0]?.path, "prayers");
    assert.equal(manifest.publishRemotes?.[1]?.url, "https://prayers.example");
  });

  it("parses a package block", () => {
    const manifest = parseManifest(`
prayfile "1"
publish "prayers", path: "prayers" do
  pray "local/project"
end
pray "local/project", path: "guidance/project"
`);
    assert.deepEqual(manifest.publishRemotes?.[0]?.packages, ["local/project"]);
  });

  it("parses a URL that ends with do", () => {
    const manifest = parseManifest(`
prayfile "1"
publish "public", "https://prayers.example/do"
`);
    assert.equal(
      manifest.publishRemotes?.[0]?.url,
      "https://prayers.example/do",
    );
    assert.deepEqual(manifest.publishRemotes?.[0]?.packages, []);
  });

  it("rejects signing_key on publish", () => {
    assert.throws(
      () =>
        parseManifest(`
prayfile "1"
publish "prayers", path: "prayers", signing_key: "secret.pem"
`),
      /does not take git:/,
    );
  });

  it("omits empty remotes from the manifest hash", () => {
    const text = `
prayfile "1"
source "local", path: "guidance"
pray "local/project"
`;
    assert.equal(
      manifestHash(parseManifest(text)),
      manifestHash(parseManifest(text)),
    );
    assert.deepEqual(parseManifest(text).publishRemotes, []);
  });
});
