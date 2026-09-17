import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { applySparseCheckout, cloneGitCache } from "./git/clone.js";
import { materializeGitCatalogFile } from "./git/materialize.js";

const UNUSED_BYTES = 128 * 1024;

describe("git blobless catalog clone", () => {
  it("keeps unused artifacts out of the clone and materializes a used file", () => {
    const root = mkdtempSync(join(tmpdir(), "pray-git-blobless-"));
    try {
      const origin = join(root, "origin");
      const clone = join(root, "clone");
      mkdirSync(join(origin, "v1/packages/sample"), { recursive: true });
      mkdirSync(join(origin, "v1/artifacts/sample/base/1.0.0"), {
        recursive: true,
      });
      mkdirSync(join(origin, "v1/artifacts/sample/heavy/1.0.0"), {
        recursive: true,
      });
      writeFileSync(join(origin, "v1/packages/sample/base.json"), "{}\n");
      const used = "v1/artifacts/sample/base/1.0.0/sample-base-1.0.0.praypkg";
      const unused =
        "v1/artifacts/sample/heavy/1.0.0/sample-heavy-1.0.0.praypkg";
      writeFileSync(join(origin, used), "used-package\n");
      writeFileSync(join(origin, unused), Buffer.alloc(UNUSED_BYTES, 7));
      runGit(origin, "init", "-b", "main");
      runGit(origin, "config", "user.name", "pray");
      runGit(origin, "config", "user.email", "pray@example.com");
      runGit(origin, "config", "uploadpack.allowFilter", "true");
      runGit(origin, "add", "-A");
      runGit(
        origin,
        "-c",
        "commit.gpgsign=false",
        "-c",
        "core.hooksPath=/dev/null",
        "commit",
        "-m",
        "catalog",
      );
      cloneGitCache(root, `file://${origin}`, clone, true);
      applySparseCheckout(clone);
      assert.equal(existsSync(join(clone, unused)), false);
      materializeGitCatalogFile(clone, used);
      assert.equal(existsSync(join(clone, used)), true);
      assert.ok(
        directoryBytes(clone) < UNUSED_BYTES,
        "blobless clone should stay smaller than the unused artifact",
      );
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});

function runGit(directory: string, ...argumentsList: string[]): void {
  const result = spawnSync("git", ["-C", directory, ...argumentsList], {
    encoding: "utf8",
    env: {
      ...process.env,
      GIT_AUTHOR_NAME: "pray",
      GIT_AUTHOR_EMAIL: "pray@example.com",
      GIT_COMMITTER_NAME: "pray",
      GIT_COMMITTER_EMAIL: "pray@example.com",
    },
  });
  assert.equal(
    result.status,
    0,
    `git ${argumentsList.join(" ")} failed: ${result.stderr ?? result.stdout}`,
  );
}

function directoryBytes(path: string): number {
  let total = 0;
  const stack = [path];
  while (stack.length > 0) {
    const current = stack.pop();
    if (current === undefined) {
      continue;
    }
    for (const name of readdirSync(current)) {
      const child = join(current, name);
      const info = statSync(child);
      if (info.isDirectory()) {
        stack.push(child);
      } else if (info.isFile()) {
        total += info.size;
      }
    }
  }
  return total;
}
