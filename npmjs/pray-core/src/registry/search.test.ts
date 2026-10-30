import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { searchLocalRegistry, searchRegistryIndexNames } from "./search.js";

describe("registry search", () => {
  it("matches names case-insensitively and reads the latest non-yanked summary", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-search-"));
    try {
      writeRegistry(root, ["sample/base", "sample/webapp"]);
      writeFileSync(
        join(root, "v1/packages/sample/base.json"),
        JSON.stringify({
          name: "sample/base",
          versions: [
            version("1.0.0", "old summary"),
            version("2.0.0", "ignored summary", true),
            version("1.1.0", "current summary"),
          ],
        }),
      );

      const hits = await searchLocalRegistry(root, "BASE", true);

      assert.deepEqual(hits, [
        { name: "sample/base", summary: "current summary" },
      ]);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it("rejects unsafe names from an untrusted index", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-search-unsafe-"));
    try {
      writeRegistry(root, ["../outside"]);
      await assert.rejects(
        searchLocalRegistry(root, "", false),
        /invalid registry package name/,
      );
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it("sorts substring matches deterministically", () => {
    assert.deepEqual(
      searchRegistryIndexNames(
        {
          spec: "prayfile-distribution-1",
          packages: [
            "sample/ä",
            "sample/webapp",
            "sample/base",
            "sample/z",
            "other/tool",
          ],
        },
        "SAMPLE",
      ),
      ["sample/base", "sample/webapp", "sample/z", "sample/ä"],
    );
  });

  it("reports malformed index JSON as an integrity error", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-search-invalid-index-"));
    try {
      mkdirSync(join(root, "v1"), { recursive: true });
      writeFileSync(join(root, "v1/index.json"), "{");

      await assert.rejects(
        searchLocalRegistry(root, "", false),
        (error: unknown) =>
          error instanceof Error &&
          error.name === "PrayError" &&
          error.message.includes("invalid distribution index"),
      );
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});

function writeRegistry(root: string, packages: string[]): void {
  mkdirSync(join(root, "v1/packages/sample"), { recursive: true });
  writeFileSync(
    join(root, "v1/index.json"),
    JSON.stringify({ spec: "prayfile-distribution-1", packages }),
  );
}

function version(number: string, summary: string, yanked = false) {
  return {
    version: number,
    artifact: `v1/artifacts/sample/base/${number}/package.praypkg`,
    yanked,
    derived_metadata: { summary },
  };
}
