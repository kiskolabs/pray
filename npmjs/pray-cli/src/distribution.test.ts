import assert from "node:assert/strict";
import { mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import {
  allowsTorrent,
  parseDistributionSettings,
  readDistributionSettings,
  writeDistributionSettings,
} from "./distribution.js";
import { PrayError } from "./errors.js";
import { initDistributionRoot } from "./publish/index.js";

describe("distribution settings", () => {
  it("treats a missing file as empty protocols", () => {
    const settings = readDistributionSettings("/no/such/root");
    assert.equal(allowsTorrent(settings), false);
  });

  it("opts in to torrent when listed", () => {
    const settings = parseDistributionSettings(
      '{"spec":"pray-distribution-config-1","protocols":["torrent"]}',
    );
    assert.equal(allowsTorrent(settings), true);
  });

  it("rejects an unknown protocol", () => {
    assert.throws(
      () =>
        parseDistributionSettings(
          '{"spec":"pray-distribution-config-1","protocols":["ipfs"]}',
        ),
      (error: unknown) =>
        error instanceof PrayError &&
        error.message.includes("unsupported protocol"),
    );
  });

  it("rejects a leftover sidecars field", () => {
    assert.throws(
      () =>
        parseDistributionSettings(
          '{"spec":"pray-distribution-config-1","sidecars":["torrent"]}',
        ),
      (error: unknown) =>
        error instanceof PrayError &&
        error.message.includes("unsupported field: sidecars"),
    );
  });

  it("writes empty protocols", () => {
    const root = mkdtempSync(join(tmpdir(), "pray-distribution-"));
    writeDistributionSettings(root);
    const settings = readDistributionSettings(root);
    assert.deepEqual(settings.protocols, []);
    assert.equal(allowsTorrent(settings), false);
  });

  it("repo init writes empty protocols", () => {
    const root = mkdtempSync(join(tmpdir(), "pray-repo-init-"));
    initDistributionRoot(root);
    const data = JSON.parse(
      readFileSync(join(root, "prayers", "v1", "distribution.json"), "utf8"),
    ) as { spec: string; protocols: string[] };
    assert.equal(data.spec, "pray-distribution-config-1");
    assert.deepEqual(data.protocols, []);
  });
});
