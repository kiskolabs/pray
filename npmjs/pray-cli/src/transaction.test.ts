import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { it } from "node:test";
import { runTransaction, writeProjectFile } from "./transaction/index.js";

it("rolls back writes and recovers a terminated writer without overwriting edits", () => {
  const root = mkdtempSync(join(tmpdir(), "pray-transaction-"));
  const path = join(root, "Prayfile");
  try {
    writeFileSync(path, "original");
    assert.throws(
      () =>
        runTransaction(root, () => {
          writeProjectFile(path, Buffer.from("candidate"));
          writeProjectFile(join(root, "output"), Buffer.from("created"));
          throw new Error("late failure");
        }),
      /late failure/,
    );
    assert.equal(readFileSync(path, "utf8"), "original");
    assert.equal(existsSync(join(root, "output")), false);
    const script = `import {runTransaction, writeProjectFile} from ${JSON.stringify(new URL("./transaction/index.js", import.meta.url).href)};
runTransaction(process.argv[1], () => { writeProjectFile(process.argv[1] + "/Prayfile", Buffer.from("intermediate")); writeProjectFile(process.argv[1] + "/Prayfile", Buffer.from("candidate")); writeProjectFile(process.argv[1] + "/output", Buffer.from("created")); process.exit(91); });`;
    const child = spawnSync(process.execPath, [
      "--input-type=module",
      "-e",
      script,
      root,
    ]);
    assert.equal(child.status, 91, child.stderr.toString());
    writeFileSync(path, "operator edit");
    assert.throws(() => runTransaction(root, () => {}), /changed/);
    assert.equal(readFileSync(path, "utf8"), "operator edit");
    renameSync(path, join(root, "operator.saved"));
    runTransaction(root, () => {});
    assert.equal(readFileSync(path, "utf8"), "original");
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

it("rejects writes from work that outlives its transaction", async () => {
  const root = mkdtempSync(join(tmpdir(), "pray-transaction-late-"));
  try {
    await new Promise<void>((complete, reject) => {
      runTransaction(root, () => {
        setImmediate(() => {
          try {
            assert.throws(
              () => writeProjectFile(join(root, "late"), "unexpected"),
              /finished/,
            );
            complete();
          } catch (error) {
            reject(error);
          }
        });
      });
    });
    assert.equal(existsSync(join(root, "late")), false);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

it("excludes a second writer while an asynchronous operation owns the project", async () => {
  const root = mkdtempSync(join(tmpdir(), "pray-transaction-concurrent-"));
  let release: () => void = () => {};
  const held = new Promise<void>((resolve) => {
    release = resolve;
  });
  try {
    const first = runTransaction(root, async () => {
      writeProjectFile(join(root, "output"), "first");
      await held;
    });
    assert.throws(
      () =>
        runTransaction(root, () =>
          writeProjectFile(join(root, "output"), "second"),
        ),
      /another pray command/,
    );
    release();
    await first;
    assert.equal(readFileSync(join(root, "output"), "utf8"), "first");
  } finally {
    release();
    rmSync(root, { recursive: true, force: true });
  }
});
