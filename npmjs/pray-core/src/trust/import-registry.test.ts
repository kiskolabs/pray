import assert from "node:assert/strict";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { importRegistryTrust } from "./import-registry.js";

describe("trust import-registry", () => {
  it("imports publisher fingerprints from a local distribution root", async () => {
    const home = mkdtempSync(join(tmpdir(), "pray-trust-home-"));
    const root = mkdtempSync(join(tmpdir(), "pray-trust-root-"));
    const previousHome = process.env.PRAY_HOME;
    process.env.PRAY_HOME = home;
    try {
      mkdirSync(join(root, "v1"), { recursive: true });
      writeFileSync(
        join(root, "v1", "ssh_publishers.json"),
        JSON.stringify({
          publishers: [
            {
              fingerprint: "sha256:deadbeef",
              id: "team-ci",
              push: true,
            },
          ],
        }),
        "utf8",
      );
      const result = await importRegistryTrust(root, undefined, false);
      assert.equal(result.publishersAdded, 1);
      assert.equal(result.hostKeysAdded, 0);
      const policyText = readFileSync(join(home, "trust.toml"), "utf8");
      assert.match(policyText, /SHA256:DEADBEEF/);
      assert.match(
        policyText,
        new RegExp(root.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")),
      );
    } finally {
      if (previousHome === undefined) {
        delete process.env.PRAY_HOME;
      } else {
        process.env.PRAY_HOME = previousHome;
      }
      rmSync(home, { recursive: true, force: true });
      rmSync(root, { recursive: true, force: true });
    }
  });
});
