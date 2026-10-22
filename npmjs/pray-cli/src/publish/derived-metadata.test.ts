import assert from "node:assert/strict";
import { cpSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { describe, it } from "node:test";
import { resolveProject } from "../resolve/project.js";
import { publishToRoot } from "./index.js";

describe("publish derived_metadata", () => {
  it("keeps historical derived_metadata when appending a version", async () => {
    const workspace = mkdtempSync(join(tmpdir(), "pray-publish-derived-"));
    const projectRoot = join(workspace, "project");
    const distributionRoot = join(workspace, "distribution");
    cpSync(
      resolve(import.meta.dirname, "../../../../examples/simple-project"),
      projectRoot,
      { recursive: true },
    );
    const manifestPath = join(projectRoot, "Prayfile");
    await publishToRoot(await resolveProject(manifestPath), distributionRoot);

    const metadataPath = join(distributionRoot, "v1/packages/sample/base.json");
    const metadata = readMetadata(metadataPath);
    const historical = {
      version: "1.0.0",
      artifact: "v1/artifacts/sample/base/1.0.0/sample-base-1.0.0.praypkg",
      yanked: false,
      targets: [] as string[],
      exports: [] as string[],
      derived_metadata: {
        summary: "historical summary",
        topics: ["cache"],
        categories: ["general"],
        possible_effects: ["improves package discovery"],
        possible_side_effects: [] as string[],
        embeddings: [
          { model: "pray-hash-bucket-v1", vector: Array(16).fill(1) },
        ],
        file_count: 2,
        character_count: 40,
        token_count: 8,
      },
    };
    metadata.versions.unshift(historical);
    writeFileSync(metadataPath, JSON.stringify(metadata, null, 2));

    const prayspecPath = join(projectRoot, "prayers/base/sample-base.prayspec");
    writeFileSync(
      prayspecPath,
      readFileSync(prayspecPath, "utf8").replace("1.4.3", "1.4.4"),
    );
    await publishToRoot(await resolveProject(manifestPath), distributionRoot);

    const rewritten = readMetadata(metadataPath);
    const historicalRow = rewritten.versions.find(
      (entry) => entry.version === "1.0.0",
    );
    assert.deepEqual(
      historicalRow?.derived_metadata,
      historical.derived_metadata,
    );
    const tip = rewritten.versions.find((entry) => entry.version === "1.4.4");
    assert.ok(tip?.derived_metadata);
    assert.ok(String(tip.derived_metadata.summary).length > 0);
  });
});

interface StoredMetadata {
  versions: Array<{
    version?: string;
    derived_metadata?: { summary: string };
  }>;
}

function readMetadata(path: string): StoredMetadata {
  return JSON.parse(readFileSync(path, "utf8")) as StoredMetadata;
}
