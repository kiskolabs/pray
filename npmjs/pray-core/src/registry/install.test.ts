import assert from "node:assert/strict";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { PrayError } from "../errors.js";
import type { ManifestPackage } from "../manifest/types.js";
import { installArtifactToCache } from "./install.js";
import type { RegistryPackageVersion } from "./types.js";

describe("registry install integrity", () => {
  it("rejects missing artifact and tree hashes before unpack", () => {
    const root = mkdtempSync(join(tmpdir(), "pray-registry-integrity-"));
    const declaration = {
      name: "sample/base",
      constraint: "1.0.0",
    } as ManifestPackage;
    const selected = {
      version: "1.0.0",
      artifact: "sample.praypkg",
      yanked: false,
      targets: [],
      exports: [],
    } as RegistryPackageVersion;
    try {
      assert.throws(
        () =>
          installArtifactToCache(
            join(root, "cache"),
            declaration,
            selected,
            Buffer.alloc(0),
          ),
        (error: unknown) =>
          error instanceof PrayError &&
          error.kind === "integrity" &&
          error.message.includes("missing artifact_hash"),
      );
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});
