import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { parseManifest } from "../manifest/index.js";
import {
  allowedPublishNames,
  pathOwnedPackageNames,
  resolvePublishDestinations,
} from "./remote.js";

describe("publish dest selection", () => {
  it("requires dest flags when no remotes are declared", () => {
    assert.throws(
      () =>
        resolvePublishDestinations(
          [],
          { to: [], roots: [], servers: [] },
          "/tmp/project",
        ),
      /--root PATH or --server URL/,
    );
  });

  it("fills declared remotes without flags", () => {
    const dests = resolvePublishDestinations(
      [
        { name: "prayers", path: "prayers", packages: [] },
        { name: "public", url: "https://prayers.example", packages: [] },
      ],
      { to: [], roots: [], servers: [] },
      "/tmp/project",
    );
    assert.equal(dests.length, 2);
    assert.equal(dests[0]?.root, "/tmp/project/prayers");
    assert.equal(dests[1]?.server, "https://prayers.example");
  });

  it("rejects an undeclared root", () => {
    assert.throws(
      () =>
        resolvePublishDestinations(
          [{ name: "prayers", path: "prayers", packages: [] }],
          { to: [], roots: ["other"], servers: [] },
          "/tmp/project",
        ),
      /not a declared publish remote/,
    );
  });

  it("publishes path packages only from a mixed manifest", () => {
    const manifest = parseManifest(`
prayfile "1"
source "local", path: "guidance"
pray "local/project"
pray "amkisko/rules", git: "https://example.com/rules.git"
`);
    assert.deepEqual(pathOwnedPackageNames(manifest), ["local/project"]);
    assert.deepEqual(allowedPublishNames(manifest, []), ["local/project"]);
  });
});
