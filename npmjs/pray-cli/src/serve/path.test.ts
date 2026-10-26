import assert from "node:assert/strict";
import {
  mkdirSync,
  mkdtempSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { it } from "node:test";
import { publicDistributionPath } from "./index.js";

it("allows only distribution files and rejects symbolic links", () => {
  const root = mkdtempSync(join(tmpdir(), "pray-serve-path-"));
  try {
    mkdirSync(join(root, "v1/artifacts"), { recursive: true });
    mkdirSync(join(root, ".pray"), { recursive: true });
    writeFileSync(join(root, ".pray/auth.db"), "secret");
    symlinkSync(join(root, ".pray/auth.db"), join(root, "v1/artifacts/leak"));

    assert.equal(publicDistributionPath(root, "/.pray/auth.db"), undefined);
    assert.equal(publicDistributionPath(root, "/v1/artifacts/leak"), undefined);
    assert.equal(
      publicDistributionPath(root, "/v1/artifacts/../../.pray/auth.db"),
      undefined,
    );
    assert.equal(
      publicDistributionPath(root, "/v1/artifacts/package.praypkg"),
      join(root, "v1/artifacts/package.praypkg"),
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
