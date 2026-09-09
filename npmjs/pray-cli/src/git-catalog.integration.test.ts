import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { runCli } from "./cli/main.js";

function createAddFixture(repo: string): void {
  mkdirSync(join(repo, "packages/base/exports"), { recursive: true });
  writeFileSync(
    join(repo, "Prayfile"),
    `prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
render mode: :managed, conflict: :fail, churn: :minimal
`,
  );
  writeFileSync(
    join(repo, "packages/base/sample-base.prayspec"),
    `Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.4.3"
  spec.summary = "shared guidance"
  spec.files = ["README.md", "exports/testing-basics.md"]
  spec.exports = {
    "testing-basics" => {
      type: "fragment",
      path: "exports/testing-basics.md",
      summary: "Testing guidance"
    }
  }
end
`,
  );
  writeFileSync(join(repo, "packages/base/README.md"), "package readme\n");
  writeFileSync(
    join(repo, "packages/base/exports/testing-basics.md"),
    "Testing guidance\n",
  );
}

function createExtraPackage(repo: string): void {
  mkdirSync(join(repo, "packages/extra/exports"), { recursive: true });
  writeFileSync(
    join(repo, "packages/extra/sample-extra.prayspec"),
    `Package::Specification.new do |spec|
  spec.name = "sample/extra"
  spec.version = "1.0.0"
  spec.summary = "extra guidance"
  spec.files = ["README.md", "exports/extra-note.md"]
  spec.exports = {
    "extra-note" => {
      type: "fragment",
      path: "exports/extra-note.md",
      summary: "Extra guidance"
    }
  }
end
`,
  );
  writeFileSync(join(repo, "packages/extra/README.md"), "extra readme\n");
  writeFileSync(
    join(repo, "packages/extra/exports/extra-note.md"),
    "Extra guidance\n",
  );
}

function runGit(directory: string, ...argumentsList: string[]): string {
  const result = spawnSync("git", ["-C", directory, ...argumentsList], {
    encoding: "utf8",
  });
  if (result.status !== 0) {
    throw new Error(
      `git ${argumentsList.join(" ")} failed: ${result.stderr ?? result.stdout ?? ""}`,
    );
  }
  return result.stdout ?? "";
}

function writeConsumerPrayfile(
  consumerRepo: string,
  distributionRepo: string,
  extra = false,
): void {
  const extraDeclaration = extra
    ? `agent "sample/extra", "~> 1.0", source: "dist"\n`
    : "";
  writeFileSync(
    join(consumerRepo, "Prayfile"),
    `prayfile "1"
source "dist", "git+file://${distributionRepo}"
agent "sample/base", "~> 1.4", source: "dist"
${extraDeclaration}target :tool_a do
  output "INSTRUCTIONS.md"
end
render mode: :managed, conflict: :fail, churn: :minimal
`,
  );
}

describe("git catalog refresh", () => {
  it("refreshes the locked catalog when a newly declared package is added", async () => {
    const workspace = mkdtempSync(join(tmpdir(), "pray-git-catalog-"));
    const previousDirectory = process.cwd();
    const previousCache = process.env.PRAY_CACHE;
    process.env.PRAY_CACHE = join(workspace, "global-cache");
    try {
      const sourceRepo = join(workspace, "source");
      const distributionRepo = join(workspace, "distribution");
      const prayersRoot = join(distributionRepo, "prayers");
      const consumerRepo = join(workspace, "consumer");
      mkdirSync(sourceRepo, { recursive: true });
      mkdirSync(distributionRepo, { recursive: true });
      mkdirSync(consumerRepo, { recursive: true });
      createAddFixture(sourceRepo);

      process.chdir(sourceRepo);
      assert.equal(
        await runCli(["add", "sample/base", "--path", "packages/base"]),
        0,
      );
      assert.equal(await runCli(["publish", "--root", prayersRoot]), 0);
      mkdirSync(prayersRoot, { recursive: true });
      runGit(distributionRepo, "init", "-b", "main");
      runGit(distributionRepo, "config", "user.name", "Pray Test");
      runGit(distributionRepo, "config", "user.email", "pray@example.com");
      runGit(distributionRepo, "config", "commit.gpgsign", "false");
      runGit(distributionRepo, "add", "-A");
      runGit(distributionRepo, "commit", "-m", "initial distribution");
      const initialRevision = runGit(
        distributionRepo,
        "rev-parse",
        "HEAD",
      ).trim();
      writeConsumerPrayfile(consumerRepo, distributionRepo);

      process.chdir(consumerRepo);
      assert.equal(await runCli(["install"]), 0);

      createExtraPackage(sourceRepo);
      process.chdir(sourceRepo);
      assert.equal(
        await runCli(["add", "sample/extra", "--path", "packages/extra"]),
        0,
      );
      assert.equal(await runCli(["publish", "--root", prayersRoot]), 0);
      runGit(distributionRepo, "add", "-A");
      runGit(distributionRepo, "commit", "-m", "publish extra package");
      const updatedRevision = runGit(
        distributionRepo,
        "rev-parse",
        "HEAD",
      ).trim();
      writeConsumerPrayfile(consumerRepo, distributionRepo, true);

      process.chdir(consumerRepo);
      const stderrChunks: string[] = [];
      const originalWrite = process.stderr.write.bind(process.stderr);
      try {
        process.stderr.write = ((chunk: string | Uint8Array) => {
          stderrChunks.push(String(chunk));
          return true;
        }) as typeof process.stderr.write;
        const lockedCode = await runCli(["install", "--locked"]);
        const lockedStderr = stderrChunks.join("");
        assert.notEqual(lockedCode, 0);
        assert.ok(lockedStderr.includes(initialRevision), lockedStderr);
        assert.ok(lockedStderr.includes("pray update"), lockedStderr);
        assert.equal(
          lockedStderr.includes("check the package name"),
          false,
          lockedStderr,
        );
      } finally {
        process.stderr.write = originalWrite;
      }

      assert.equal(await runCli(["install"]), 0);
      const lockfile = readFileSync(
        join(consumerRepo, "Prayfile.lock"),
        "utf8",
      );
      assert.ok(lockfile.includes(updatedRevision), lockfile);
      assert.equal(lockfile.includes(initialRevision), false, lockfile);
      assert.ok(lockfile.includes("sample/extra"));
    } finally {
      process.chdir(previousDirectory);
      if (previousCache === undefined) delete process.env.PRAY_CACHE;
      else process.env.PRAY_CACHE = previousCache;
      rmSync(workspace, { recursive: true, force: true });
    }
  });
});
