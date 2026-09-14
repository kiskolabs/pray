import assert from "node:assert/strict";
import { cpSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { describe, it } from "node:test";
import { resolveProject } from "../resolve/project.js";
import { publishToRoot } from "./index.js";

describe("publish", () => {
  it("preserves publish metadata until package content changes", async () => {
    const workspace = mkdtempSync(join(tmpdir(), "pray-publish-"));
    const projectRoot = join(workspace, "project");
    const distributionRoot = join(workspace, "distribution");
    cpSync(
      resolve(import.meta.dirname, "../../../../examples/simple-project"),
      projectRoot,
      { recursive: true },
    );
    const manifestPath = join(projectRoot, "Prayfile");
    const project = await resolveProject(manifestPath);

    await publishToRoot(project, distributionRoot);
    const metadataPath = join(distributionRoot, "v1/packages/sample/base.json");
    const metadata = readMetadata(metadataPath);
    const initial = metadata.versions[0];
    assert.ok(initial);
    initial.published_at = "2020-01-01T00:00:00.123Z";
    initial.yanked = true;
    writeFileSync(metadataPath, JSON.stringify(metadata, null, 2));

    await publishToRoot(project, distributionRoot);
    const unchangedMetadata = readMetadata(metadataPath);
    const unchanged = unchangedMetadata.versions[0];
    assert.ok(unchanged);
    assert.equal(unchanged.published_at, 1_577_836_800);
    assert.equal(unchanged.yanked, true);
    await publishToRoot(project, distributionRoot);
    assert.deepEqual(readMetadata(metadataPath), unchangedMetadata);

    await publishToRoot(project, distributionRoot, "replacement");
    const resigned = readMetadata(metadataPath).versions[0];
    assert.ok(resigned);
    assert.equal(resigned.signer, "replacement");
    assert.equal(resigned.published_at, 1_577_836_800);
    assert.equal(resigned.yanked, true);

    const prayspecPath = join(
      projectRoot,
      "packages/base/sample-base.prayspec",
    );
    writeFileSync(
      prayspecPath,
      readFileSync(prayspecPath, "utf8").replace(
        "small guidance bundle",
        "revised guidance bundle",
      ),
    );
    await publishToRoot(await resolveProject(manifestPath), distributionRoot);
    const specificationChanged = readMetadata(metadataPath).versions[0];
    assert.ok(specificationChanged);
    assert.notEqual(specificationChanged.artifact_hash, resigned.artifact_hash);
    assert.notEqual(specificationChanged.published_at, 1_577_836_800);
    assert.equal(specificationChanged.yanked, true);

    writeFileSync(
      join(projectRoot, "packages/base/README.md"),
      "Changed package\n",
    );
    await publishToRoot(await resolveProject(manifestPath), distributionRoot);
    const changed = readMetadata(metadataPath).versions[0];
    assert.ok(changed);
    assert.notEqual(changed.artifact_hash, specificationChanged.artifact_hash);
  });
});

interface StoredMetadata {
  versions: Array<{
    artifact_hash?: string;
    published_at?: number | string;
    signer?: string;
    yanked: boolean;
  }>;
}

function readMetadata(path: string): StoredMetadata {
  return JSON.parse(readFileSync(path, "utf8")) as StoredMetadata;
}
