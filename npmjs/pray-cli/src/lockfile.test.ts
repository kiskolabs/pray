import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "node:test";
import { parse } from "smol-toml";
import { PrayError } from "./errors.js";
import {
  buildLockfile,
  lockfilesEquivalent,
  parseLockfile,
  readLockfile,
  serializeLockfile,
} from "./lockfile/index.js";
import { parseLockfileValue } from "./lockfile/parse.js";
import { canonicalLockfile, type Lockfile } from "./lockfile/types.js";

describe("lockfile", () => {
  it("parses lockfile text without reading from disk", () => {
    const lockfile = parseLockfile(`
prayfile_lock = "1"
spec = "0.1"
generated_by = "pray test"
manifest_hash = "sha256:abc"
source = []
package = []
target = []
managed_span = []
`);
    assert.equal(lockfile.manifest_hash, "sha256:abc");
  });

  it("serializes fixture lockfile in canonical pretty format", () => {
    const fixturePath = resolve(
      import.meta.dirname,
      "../../../examples/simple-project/Prayfile.lock",
    );
    const expected = readFileSync(fixturePath, "utf8");
    const lockfile = parseLockfile(expected);
    assert.equal(serializeLockfile(lockfile), expected);
  });

  it("parses a valid fixture lockfile", () => {
    const fixturePath = resolve(
      import.meta.dirname,
      "../../../examples/simple-project/Prayfile.lock",
    );
    const lockfile = readLockfile(fixturePath);
    assert.equal(lockfile.prayfile_lock, "1");
    assert.equal(lockfile.package[0]?.name, "sample/base");
    assert.equal(lockfile.managed_span.length, 2);
  });

  it("rejects malformed lockfile values", () => {
    assert.throws(
      () => parseLockfileValue({ prayfile_lock: "1" }),
      (error: unknown) =>
        error instanceof PrayError &&
        error.kind === "parse" &&
        error.message.includes("lockfile"),
    );
  });

  it("compares lockfiles by canonical serialization", () => {
    const fixturePath = resolve(
      import.meta.dirname,
      "../../../examples/simple-project/Prayfile.lock",
    );
    const parsed = parseLockfileValue(
      parse(readFileSync(fixturePath, "utf8")),
    );
    const reordered: Lockfile = {
      ...parsed,
      package: [...parsed.package].reverse(),
      source: [...parsed.source].reverse(),
    };
    assert.equal(lockfilesEquivalent(parsed, reordered), true);
    assert.equal(
      serializeLockfile(canonicalLockfile(parsed)),
      serializeLockfile(reordered),
    );
  });

  it("serializes optional environment field", () => {
    const lockfile = buildLockfile({
      manifestHash: "sha256:abc",
      environment: "development",
      projectRoot: "/tmp/project",
      manifestSources: [],
      manifestTargets: [],
      rendered: [],
      packages: [],
    });
    assert.match(serializeLockfile(lockfile), /environment = "development"/);
  });

  it("detects lockfile changes", () => {
    const left = buildLockfile({
      manifestHash: "sha256:abc",
      projectRoot: "/tmp/project",
      manifestSources: [],
      manifestTargets: [],
      rendered: [],
      packages: [],
    });
    const right = {
      ...left,
      manifest_hash: "sha256:def",
    };
    assert.equal(lockfilesEquivalent(left, right), false);
  });
});
