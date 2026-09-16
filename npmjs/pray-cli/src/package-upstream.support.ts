import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
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

export function runGit(directory: string, ...argumentsList: string[]): string {
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
  runGit(distributionRepo, "config", "commit.gpgsign", "false");
  runGit(distributionRepo, "add", "-A");
  runGit(distributionRepo, "commit", "-m", "initial distribution");
}

function writeForkPackage(
  catalog: string,
  source: string,
  constraint: string,
): void {
  const root = join(catalog, "packages/fork-base");
  mkdirSync(join(root, "exports"), { recursive: true });
  copyFileSync(
    join(source, "packages/base/README.md"),
    join(root, "README.md"),
  );
  copyFileSync(
    join(source, "packages/base/exports/testing-basics.md"),
    join(root, "exports/testing-basics.md"),
  );
  writeFileSync(
    join(root, "fork-base.prayspec"),
    `Package::Specification.new do |spec|
  spec.name = "fork/base"
  spec.version = "1.0.0"
  spec.summary = "forked guidance"
  spec.files = ["README.md", "exports/testing-basics.md"]
  spec.exports = {
    "testing-basics" => {
      type: "fragment",
      path: "exports/testing-basics.md",
      summary: "Testing guidance"
    }
  }
  spec.upstream "sample/base", "${constraint}"
end
`,
  );
}

function writeForkPrayfile(catalog: string, distribution: string): void {
  writeFileSync(
    join(catalog, "Prayfile"),
    `prayfile "1"
source "sample", "git+file://${distribution}"
target :tool_a do
  output "INSTRUCTIONS.md"
end
agent "fork/base", "~> 1.0", path: "packages/fork-base"
render mode: :managed, conflict: :fail, churn: :minimal
`,
  );
}

export function bumpUpstreamVersion(source: string): void {
  writeFileSync(
    join(source, "packages/base/sample-base.prayspec"),
    `Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.4.4"
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
  writeFileSync(
    join(source, "packages/base/exports/testing-basics.md"),
    "Testing guidance v2\n",
  );
}

export function writeEmptyForkPackage(
  catalog: string,
  constraint: string,
): void {
  const root = join(catalog, "packages/fork-base");
  mkdirSync(root, { recursive: true });
  writeFileSync(
    join(root, "fork-base.prayspec"),
    `Package::Specification.new do |spec|
  spec.name = "fork/base"
  spec.version = "1.0.0"
  spec.summary = "forked guidance"
  spec.files = []
  spec.upstream "sample/base", "${constraint}"
end
`,
  );
}

export async function publishUpstreamCatalog(
  workspace: string,
): Promise<{ catalog: string; source: string }> {
  const sourceRepo = join(workspace, "source");
  const distributionRepo = join(workspace, "distribution");
  const prayersRoot = join(distributionRepo, "prayers");
  const catalogRepo = join(workspace, "catalog");
  mkdirSync(sourceRepo, { recursive: true });
  mkdirSync(distributionRepo, { recursive: true });
  mkdirSync(catalogRepo, { recursive: true });
  createAddFixture(sourceRepo);
  process.chdir(sourceRepo);
  assert.equal(
    await runCli(["add", "sample/base", "--path", "packages/base"]),
    0,
  );
  assert.equal(await runCli(["publish", "--root", prayersRoot]), 0);
  initDistributionRepo(distributionRepo, prayersRoot);
  writeForkPrayfile(catalogRepo, distributionRepo);
  return { catalog: catalogRepo, source: sourceRepo };
}

export async function catalogAfterUpstreamBump(
  workspace: string,
  constraint: string,
): Promise<string> {
  const sourceRepo = join(workspace, "source");
  const distributionRepo = join(workspace, "distribution");
  const prayersRoot = join(distributionRepo, "prayers");
  const catalogRepo = join(workspace, "catalog");
  mkdirSync(sourceRepo, { recursive: true });
  mkdirSync(distributionRepo, { recursive: true });
  mkdirSync(catalogRepo, { recursive: true });
  createAddFixture(sourceRepo);
  process.chdir(sourceRepo);
  assert.equal(
    await runCli(["add", "sample/base", "--path", "packages/base"]),
    0,
  );
  assert.equal(await runCli(["publish", "--root", prayersRoot]), 0);
  initDistributionRepo(distributionRepo, prayersRoot);
  writeForkPackage(catalogRepo, sourceRepo, constraint);
  writeForkPrayfile(catalogRepo, distributionRepo);
  process.chdir(catalogRepo);
  assert.equal(await runCli(["install"]), 0);
  bumpUpstreamVersion(sourceRepo);
  process.chdir(sourceRepo);
  assert.equal(await runCli(["publish", "--root", prayersRoot]), 0);
  runGit(distributionRepo, "add", "-A");
  runGit(distributionRepo, "commit", "-m", "publish 1.4.4");
  return catalogRepo;
}

export async function withWorkspace(
  callback: (workspace: string) => Promise<void>,
): Promise<void> {
  const workspace = mkdtempSync(join(tmpdir(), "pray-package-upstream-"));
  const previousDirectory = process.cwd();
  const previousCache = process.env.PRAY_CACHE;
  process.env.PRAY_CACHE = join(workspace, "global-cache");
  try {
    await callback(workspace);
  } finally {
    process.chdir(previousDirectory);
    if (previousCache === undefined) delete process.env.PRAY_CACHE;
    else process.env.PRAY_CACHE = previousCache;
    rmSync(workspace, { recursive: true, force: true });
  }
}
