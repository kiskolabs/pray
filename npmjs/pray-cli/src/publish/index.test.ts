import assert from "node:assert/strict";
import {
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
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

    // A second publisher over unchanged packages must not restate authorship of bytes
    // nobody rebuilt, or one maintainer's release buries itself in the other's identity.
    await publishToRoot(project, distributionRoot, "replacement");
    assert.deepEqual(readMetadata(metadataPath), unchangedMetadata);
    const preserved = unchangedMetadata.versions[0];
    assert.ok(preserved);

    const prayspecPath = join(projectRoot, "prayers/base/sample-base.prayspec");
    writeFileSync(
      prayspecPath,
      readFileSync(prayspecPath, "utf8").replace(
        "small guidance bundle",
        "revised guidance bundle",
      ),
    );
    await publishToRoot(
      await resolveProject(manifestPath),
      distributionRoot,
      "replacement",
    );
    const specificationChanged = readMetadata(metadataPath).versions[0];
    assert.ok(specificationChanged);
    assert.equal(specificationChanged.signer, "replacement");
    assert.notEqual(
      specificationChanged.artifact_hash,
      preserved.artifact_hash,
    );
    assert.notEqual(specificationChanged.published_at, 1_577_836_800);
    assert.equal(specificationChanged.yanked, true);

    writeFileSync(
      join(projectRoot, "prayers/base/README.md"),
      "Changed package\n",
    );
    await publishToRoot(await resolveProject(manifestPath), distributionRoot);
    const changed = readMetadata(metadataPath).versions[0];
    assert.ok(changed);
    assert.notEqual(changed.artifact_hash, specificationChanged.artifact_hash);
  });

  it("writes a torrent descriptor when the root lists torrent", async () => {
    const workspace = mkdtempSync(join(tmpdir(), "pray-publish-torrent-"));
    const projectRoot = join(workspace, "project");
    const distributionRoot = join(workspace, "distribution");
    cpSync(
      resolve(import.meta.dirname, "../../../../examples/simple-project"),
      projectRoot,
      { recursive: true },
    );
    mkdirSync(join(distributionRoot, "v1"), { recursive: true });
    writeFileSync(
      join(distributionRoot, "v1/distribution.json"),
      JSON.stringify({
        spec: "pray-distribution-config-1",
        protocols: ["torrent"],
        bootstrap_trackers: ["http://tracker.example/announce"],
      }),
    );
    await publishToRoot(
      await resolveProject(join(projectRoot, "Prayfile")),
      distributionRoot,
    );

    const artifact = join(
      distributionRoot,
      "v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg",
    );
    const descriptor = `${artifact}.praytorrent.json`;
    assert.equal(existsSync(descriptor), true);
    const manifest = JSON.parse(readFileSync(descriptor, "utf8")) as {
      spec: string;
      name: string;
      version: string;
      artifact_url: string;
      artifact_hash: string;
      pieces: string[];
      sources: string[];
      trackers: string[];
    };
    assert.equal(manifest.spec, "pray-torrent-v1");
    assert.equal(manifest.name, "sample/base");
    assert.equal(manifest.version, "1.4.3");
    assert.equal(
      manifest.artifact_url,
      "v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg",
    );
    assert.ok(manifest.artifact_hash.startsWith("sha256:"));
    assert.ok(manifest.pieces.length > 0);
    assert.ok(
      manifest.sources.includes(
        "v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg",
      ),
    );
    assert.deepEqual(manifest.trackers, ["http://tracker.example/announce"]);
  });

  it("skips the torrent descriptor when protocols are empty", async () => {
    const workspace = mkdtempSync(join(tmpdir(), "pray-publish-no-torrent-"));
    const projectRoot = join(workspace, "project");
    const distributionRoot = join(workspace, "distribution");
    cpSync(
      resolve(import.meta.dirname, "../../../../examples/simple-project"),
      projectRoot,
      { recursive: true },
    );
    await publishToRoot(
      await resolveProject(join(projectRoot, "Prayfile")),
      distributionRoot,
    );
    const artifact = join(
      distributionRoot,
      "v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg",
    );
    assert.equal(existsSync(artifact), true);
    assert.equal(existsSync(`${artifact}.praytorrent.json`), false);
  });

  it("writes a missing torrent descriptor on republish after torrent is listed", async () => {
    const workspace = mkdtempSync(
      join(tmpdir(), "pray-publish-torrent-later-"),
    );
    const projectRoot = join(workspace, "project");
    const distributionRoot = join(workspace, "distribution");
    cpSync(
      resolve(import.meta.dirname, "../../../../examples/simple-project"),
      projectRoot,
      { recursive: true },
    );
    const project = await resolveProject(join(projectRoot, "Prayfile"));
    await publishToRoot(project, distributionRoot);
    mkdirSync(join(distributionRoot, "v1"), { recursive: true });
    writeFileSync(
      join(distributionRoot, "v1/distribution.json"),
      JSON.stringify({
        spec: "pray-distribution-config-1",
        protocols: ["torrent"],
      }),
    );
    await publishToRoot(project, distributionRoot);
    const artifact = join(
      distributionRoot,
      "v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg",
    );
    assert.equal(existsSync(`${artifact}.praytorrent.json`), true);
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
