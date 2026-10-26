import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { PrayError } from "../errors.js";
import {
  registryArtifactSignature,
  verifyRegistrySignature,
} from "./signature.js";
import type { RegistryPackageVersion } from "./types.js";

describe("registry signatures", () => {
  it("accepts a matching signature and rejects a mismatch", () => {
    const artifact = Buffer.from("package");
    const selected = {
      version: "1.0.0",
      artifact: "v1/artifacts/sample.praypkg",
      treeHash: "sha256:tree",
      signer: "local",
      yanked: false,
      targets: [],
      exports: [],
      signature: registryArtifactSignature(artifact, "sha256:tree", "local"),
    } as RegistryPackageVersion;

    verifyRegistrySignature("sample/base", selected, artifact);

    selected.signature = "sha256:deadbeef";
    assert.throws(
      () => verifyRegistrySignature("sample/base", selected, artifact),
      (error: unknown) =>
        error instanceof PrayError &&
        error.message.includes("signature mismatch"),
    );
  });
});
