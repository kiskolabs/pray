import assert from "node:assert/strict";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, beforeEach, describe, it } from "node:test";
import { cleanUnusedRegistryCache } from "./index.js";

describe("unused registry cache cleaning", () => {
  let projectRoot = "";

  beforeEach(() => {
    projectRoot = mkdtempSync(join(tmpdir(), "pray-clean-unused-"));
  });

  afterEach(() => {
    rmSync(projectRoot, { recursive: true, force: true });
  });

  function createCache(relativePath: string): string {
    const path = join(projectRoot, relativePath);
    mkdirSync(path, { recursive: true });
    writeFileSync(join(path, "entry"), "cached");
    return path;
  }

  function writeLockfile(packagePath: string): void {
    writeFileSync(
      join(projectRoot, "Prayfile.lock"),
      `prayfile_lock = "1"
spec = "0.1"
generated_by = "pray test"
manifest_hash = "sha256:0000000000000000000000000000000000000000000000000000000000000000"
source = []
target = []
managed_span = []
provisioned = []

[[package]]
name = "sample/base"
version = "1.4.3"
path = "${packagePath}"
tree_hash = "sha256:1111111111111111111111111111111111111111111111111111111111111111"
artifact_hash = "sha256:2222222222222222222222222222222222222222222222222222222222222222"
artifact = "path:${packagePath}"
exports = []
dependencies = []
`,
    );
  }

  it("retains only locked entries without touching other state", () => {
    const locked = createCache(".pray/cache/registry/sample/base/1.4.3/source");
    const staleVersion = createCache(
      ".pray/cache/registry/sample/base/1.4.2/source",
    );
    const staleSource = createCache(
      ".pray/cache/registry/sample/base/1.4.3/other",
    );
    const legacy = createCache(".pray/cache/registry/legacy/sample/base/1.4.3");
    const staging = createCache(
      ".pray/cache/registry/sample/base/1.4.3/source.staging",
    );
    const gitCache = createCache(".pray/cache/git/repository");
    const vendor = createCache(".pray/vendor/sample-base");
    writeFileSync(join(projectRoot, ".pray/state.json"), "{}");
    const globalCache = createCache("global/cache/entry");
    writeLockfile("./.pray/cache/registry/sample/base/1.4.3/source");

    cleanUnusedRegistryCache(projectRoot);

    assert.ok(existsSync(locked));
    assert.ok(!existsSync(staleVersion));
    assert.ok(!existsSync(staleSource));
    assert.ok(!existsSync(legacy));
    assert.ok(!existsSync(staging));
    assert.ok(existsSync(gitCache));
    assert.ok(existsSync(vendor));
    assert.ok(existsSync(join(projectRoot, ".pray/state.json")));
    assert.ok(existsSync(globalCache));
  });

  it("requires a readable and parseable lockfile", () => {
    for (const contents of [undefined, "not valid = ["]) {
      rmSync(join(projectRoot, "Prayfile.lock"), { force: true });
      const cache = createCache(
        ".pray/cache/registry/sample/base/1.0.0/source",
      );
      if (contents !== undefined) {
        writeFileSync(join(projectRoot, "Prayfile.lock"), contents);
      }
      assert.throws(() => cleanUnusedRegistryCache(projectRoot));
      assert.ok(existsSync(cache));
    }
  });

  it("rejects an incomplete lockfile before deleting", () => {
    const cache = createCache(".pray/cache/registry/sample/base/1.0.0/source");
    writeLockfile("./.pray/cache/registry/sample/base/1.0.0/source");
    const lockfilePath = join(projectRoot, "Prayfile.lock");
    const contents = readFileSync(lockfilePath, "utf8").replace(
      "sha256:0000000000000000000000000000000000000000000000000000000000000000",
      "sha256:incomplete",
    );
    writeFileSync(lockfilePath, contents);

    assert.throws(() => cleanUnusedRegistryCache(projectRoot), /manifest_hash/);
    assert.ok(existsSync(cache));
  });

  it("does not follow registry symlinks", () => {
    const outside = mkdtempSync(join(tmpdir(), "pray-clean-outside-"));
    writeFileSync(join(outside, "keep"), "outside");
    mkdirSync(join(projectRoot, ".pray/cache/registry"), { recursive: true });
    symlinkSync(
      outside,
      join(projectRoot, ".pray/cache/registry/stale"),
      "dir",
    );
    writeLockfile("./packages/base");

    cleanUnusedRegistryCache(projectRoot);

    assert.ok(!existsSync(join(projectRoot, ".pray/cache/registry/stale")));
    assert.ok(existsSync(join(outside, "keep")));
    rmSync(outside, { recursive: true, force: true });
  });
});
