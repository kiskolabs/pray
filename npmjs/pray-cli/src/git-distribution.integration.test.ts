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
): void {
  writeFileSync(
    join(consumerRepo, "Prayfile"),
    `prayfile "1"
source "dist", "git+file://${distributionRepo}"
agent "sample/base", "~> 1.4", source: "dist"
target :tool_a do
  output "INSTRUCTIONS.md"
end
render mode: :managed, conflict: :fail, churn: :minimal
`,
  );
}

async function withWorkspace(
  callback: (workspace: string) => Promise<void>,
): Promise<void> {
  const workspace = mkdtempSync(join(tmpdir(), "pray-git-distribution-"));
  const previousDirectory = process.cwd();
  try {
    await callback(workspace);
  } finally {
    process.chdir(previousDirectory);
    rmSync(workspace, { recursive: true, force: true });
  }
}

describe("git distribution install", () => {
  it("resolves packages from a git distribution repository", async () => {
    await withWorkspace(async (workspace) => {
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

      initDistributionRepo(distributionRepo, prayersRoot);
      writeConsumerPrayfile(consumerRepo, distributionRepo);

      process.chdir(consumerRepo);
      assert.equal(await runCli(["install"]), 0);

      const lockfile = readFileSync(
        join(consumerRepo, "Prayfile.lock"),
        "utf8",
      );
      assert.match(lockfile, /sample\/base/);
      assert.match(lockfile, /revision = "/);
      assert.match(
        readFileSync(join(consumerRepo, "INSTRUCTIONS.md"), "utf8"),
        /<!-- pray:/,
      );
    });
  });

  it("keeps the locked git revision when the distribution moves forward", async () => {
    await withWorkspace(async (workspace) => {
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
      initDistributionRepo(distributionRepo, prayersRoot);

      const initialRevision = runGit(
        distributionRepo,
        "rev-parse",
        "HEAD",
      ).trim();
      writeConsumerPrayfile(consumerRepo, distributionRepo);

      process.chdir(consumerRepo);
      assert.equal(await runCli(["install"]), 0);

      const indexPath = join(prayersRoot, "v1", "index.json");
      writeFileSync(
        indexPath,
        readFileSync(indexPath, "utf8").replace(
          '"packages"',
          '"marker":"moved-forward","packages"',
        ),
      );
      runGit(distributionRepo, "add", "-A");
      runGit(distributionRepo, "commit", "-m", "advance distribution");

      assert.equal(await runCli(["install", "--locked"]), 0);

      const lockfile = readFileSync(
        join(consumerRepo, "Prayfile.lock"),
        "utf8",
      );
      assert.ok(lockfile.includes(initialRevision));
    });
  });

  it("advances the git revision on update", async () => {
    await withWorkspace(async (workspace) => {
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
      initDistributionRepo(distributionRepo, prayersRoot);
      writeConsumerPrayfile(consumerRepo, distributionRepo);

      process.chdir(consumerRepo);
      assert.equal(await runCli(["install"]), 0);

      const indexPath = join(prayersRoot, "v1", "index.json");
      writeFileSync(
        indexPath,
        readFileSync(indexPath, "utf8").replace(
          '"packages"',
          '"marker":"update-target","packages"',
        ),
      );
      runGit(distributionRepo, "add", "-A");
      runGit(distributionRepo, "commit", "-m", "advance distribution");
      const advancedRevision = runGit(
        distributionRepo,
        "rev-parse",
        "HEAD",
      ).trim();

      assert.equal(await runCli(["update"]), 0);

      const lockfile = readFileSync(
        join(consumerRepo, "Prayfile.lock"),
        "utf8",
      );
      assert.ok(lockfile.includes(advancedRevision));
    });
  });
});
