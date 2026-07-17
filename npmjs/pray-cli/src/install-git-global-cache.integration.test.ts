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

function initDistributionRepo(
  distributionRepo: string,
  prayersRoot: string,
): void {
  mkdirSync(prayersRoot, { recursive: true });
  runGit(distributionRepo, "init", "-b", "main");
  runGit(distributionRepo, "config", "user.name", "Pray Test");
  runGit(distributionRepo, "config", "user.email", "pray@example.com");
  runGit(distributionRepo, "add", "-A");
  runGit(distributionRepo, "commit", "-m", "initial distribution");
}

function writeConsumerPrayfile(
  consumerRepo: string,
  distributionRepo: string,
  constraint = "~> 1.4",
): void {
  writeFileSync(
    join(consumerRepo, "Prayfile"),
    `prayfile "1"
source "dist", "git+file://${distributionRepo}"
agent "sample/base", "${constraint}", source: "dist"
target :tool_a do
  output "INSTRUCTIONS.md"
end
render mode: :managed, conflict: :fail, churn: :minimal
`,
  );
}

async function runCliWithCache(
  cacheRoot: string,
  argumentsList: string[],
): Promise<number> {
  const previousCache = process.env.PRAY_CACHE;
  process.env.PRAY_CACHE = cacheRoot;
  try {
    return await runCli(argumentsList);
  } finally {
    if (previousCache === undefined) {
      delete process.env.PRAY_CACHE;
    } else {
      process.env.PRAY_CACHE = previousCache;
    }
  }
}

describe("global git cache install", () => {
  it("refreshes the stale global mirror when constraints require newer versions", async () => {
    const workspace = mkdtempSync(join(tmpdir(), "pray-global-git-cache-"));
    const globalCache = join(workspace, "global-cache");
    const sourceRepo = join(workspace, "source");
    const distributionRepo = join(workspace, "distribution");
    const prayersRoot = join(distributionRepo, "prayers");
    const firstConsumer = join(workspace, "first-consumer");
    const secondConsumer = join(workspace, "second-consumer");
    const previousDirectory = process.cwd();

    try {
      mkdirSync(sourceRepo, { recursive: true });
      mkdirSync(distributionRepo, { recursive: true });
      mkdirSync(firstConsumer, { recursive: true });
      mkdirSync(secondConsumer, { recursive: true });
      createAddFixture(sourceRepo);

      process.chdir(sourceRepo);
      assert.equal(
        await runCli(["add", "sample/base", "--path", "packages/base"]),
        0,
      );
      assert.equal(await runCli(["publish", "--root", prayersRoot]), 0);
      initDistributionRepo(distributionRepo, prayersRoot);

      writeConsumerPrayfile(firstConsumer, distributionRepo);
      process.chdir(firstConsumer);
      assert.equal(await runCliWithCache(globalCache, ["install"]), 0);

      writeFileSync(
        join(sourceRepo, "packages/base/sample-base.prayspec"),
        `Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "2.0.0"
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
      process.chdir(sourceRepo);
      assert.equal(await runCli(["publish", "--root", prayersRoot]), 0);
      runGit(distributionRepo, "add", "-A");
      runGit(distributionRepo, "commit", "-m", "publish major version");
      const updatedRevision = runGit(
        distributionRepo,
        "rev-parse",
        "HEAD",
      ).trim();

      writeConsumerPrayfile(secondConsumer, distributionRepo, "~> 2.0");
      process.chdir(secondConsumer);
      assert.equal(await runCliWithCache(globalCache, ["install"]), 0);

      const lockfile = readFileSync(
        join(secondConsumer, "Prayfile.lock"),
        "utf8",
      );
      assert.ok(lockfile.includes(updatedRevision));
      assert.ok(lockfile.includes("2.0.0"));
    } finally {
      process.chdir(previousDirectory);
      rmSync(workspace, { recursive: true, force: true });
    }
  });
});
