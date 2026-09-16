import assert from "node:assert/strict";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { describe, it } from "node:test";
import { runCli } from "./cli/main.js";
import {
  bumpUpstreamVersion,
  catalogAfterUpstreamBump,
  publishUpstreamCatalog,
  runGit,
  withWorkspace,
  writeEmptyForkPackage,
} from "./package-upstream.support.js";

describe("package upstream refresh", () => {
  it("replaces a clean fork from the locked upstream on update", async () => {
    await withWorkspace(async (workspace) => {
      const catalog = await catalogAfterUpstreamBump(workspace, "~> 1.4");
      process.chdir(catalog);
      assert.equal(await runCli(["update"]), 0);
      assert.equal(
        readFileSync(
          join(catalog, "packages/fork-base/exports/testing-basics.md"),
          "utf8",
        ),
        "Testing guidance v2\n",
      );
      const spec = readFileSync(
        join(catalog, "packages/fork-base/fork-base.prayspec"),
        "utf8",
      );
      assert.match(spec, /fork\/base/);
      assert.match(spec, /1.0.0/);
      const lockfile = readFileSync(join(catalog, "Prayfile.lock"), "utf8");
      assert.match(lockfile, /1\.4\.4/);
      assert.match(spec, /exports\/testing-basics\.md/);
      assert.equal(await runCli(["package"]), 0);
      assert.equal(
        existsSync(join(catalog, ".pray/packages/fork-base-1.0.0.praypkg")),
        true,
      );
    });
  });

  it("keeps an exact upstream pin until --latest", async () => {
    await withWorkspace(async (workspace) => {
      const catalog = await catalogAfterUpstreamBump(workspace, "= 1.4.3");
      process.chdir(catalog);
      assert.equal(await runCli(["update"]), 0);
      assert.equal(
        readFileSync(
          join(catalog, "packages/fork-base/exports/testing-basics.md"),
          "utf8",
        ),
        "Testing guidance\n",
      );
      assert.match(
        readFileSync(
          join(catalog, "packages/fork-base/fork-base.prayspec"),
          "utf8",
        ),
        /= 1\.4\.3/,
      );
      const locked = readFileSync(join(catalog, "Prayfile.lock"), "utf8");
      assert.match(locked, /1\.4\.3/);
      assert.doesNotMatch(locked, /1\.4\.4/);

      assert.equal(await runCli(["update", "--latest"]), 0);
      assert.equal(
        readFileSync(
          join(catalog, "packages/fork-base/exports/testing-basics.md"),
          "utf8",
        ),
        "Testing guidance v2\n",
      );
      assert.match(
        readFileSync(
          join(catalog, "packages/fork-base/fork-base.prayspec"),
          "utf8",
        ),
        /= 1\.4\.4/,
      );
      assert.match(
        readFileSync(join(catalog, "Prayfile.lock"), "utf8"),
        /1\.4\.4/,
      );
    });
  });

  it("does not rewrite an upstream pin on --latest --dry-run", async () => {
    await withWorkspace(async (workspace) => {
      const catalog = await catalogAfterUpstreamBump(workspace, "= 1.4.3");
      process.chdir(catalog);
      const chunks: string[] = [];
      const previousWrite = process.stdout.write.bind(process.stdout);
      process.stdout.write = ((chunk: string | Uint8Array) => {
        chunks.push(
          typeof chunk === "string"
            ? chunk
            : Buffer.from(chunk).toString("utf8"),
        );
        return true;
      }) as typeof process.stdout.write;
      try {
        assert.equal(await runCli(["update", "--latest", "--dry-run"]), 0);
      } finally {
        process.stdout.write = previousWrite;
      }
      const stdout = chunks.join("");
      assert.match(stdout, /fork\/base upstream/);
      assert.match(
        readFileSync(
          join(catalog, "packages/fork-base/fork-base.prayspec"),
          "utf8",
        ),
        /= 1\.4\.3/,
      );
      assert.equal(
        readFileSync(
          join(catalog, "packages/fork-base/exports/testing-basics.md"),
          "utf8",
        ),
        "Testing guidance\n",
      );
    });
  });

  it("copies an empty path fork from upstream on install", async () => {
    await withWorkspace(async (workspace) => {
      const { catalog } = await publishUpstreamCatalog(workspace);
      writeEmptyForkPackage(catalog, "~> 1.4");
      process.chdir(catalog);
      assert.equal(await runCli(["install"]), 0);
      assert.equal(
        readFileSync(
          join(catalog, "packages/fork-base/exports/testing-basics.md"),
          "utf8",
        ),
        "Testing guidance\n",
      );
      const spec = readFileSync(
        join(catalog, "packages/fork-base/fork-base.prayspec"),
        "utf8",
      );
      assert.match(spec, /fork\/base/);
      assert.match(spec, /exports\/testing-basics.md/);
    });
  });

  it("keeps an overlay file when upstream moves", async () => {
    await withWorkspace(async (workspace) => {
      const { catalog, source } = await publishUpstreamCatalog(workspace);
      writeEmptyForkPackage(catalog, "~> 1.4");
      process.chdir(catalog);
      assert.equal(await runCli(["install"]), 0);
      mkdirSync(join(catalog, "packages/fork-base/overlays"), {
        recursive: true,
      });
      writeFileSync(
        join(catalog, "packages/fork-base/overlays/note.md"),
        "local overlay\n",
      );
      const specPath = join(catalog, "packages/fork-base/fork-base.prayspec");
      writeFileSync(
        specPath,
        readFileSync(specPath, "utf8").replace(
          '"README.md"',
          '"README.md", "overlays/note.md"',
        ),
      );
      assert.equal(await runCli(["install"]), 0);
      bumpUpstreamVersion(source);
      const prayersRoot = join(workspace, "distribution/prayers");
      process.chdir(source);
      assert.equal(await runCli(["publish", "--root", prayersRoot]), 0);
      runGit(join(workspace, "distribution"), "add", "-A");
      runGit(join(workspace, "distribution"), "commit", "-m", "publish 1.4.4");
      process.chdir(catalog);
      assert.equal(await runCli(["update"]), 0);
      assert.equal(
        readFileSync(
          join(catalog, "packages/fork-base/overlays/note.md"),
          "utf8",
        ),
        "local overlay\n",
      );
      assert.equal(
        readFileSync(
          join(catalog, "packages/fork-base/exports/testing-basics.md"),
          "utf8",
        ),
        "Testing guidance v2\n",
      );
    });
  });

  it("lists path-fork file drift on outdated", async () => {
    await withWorkspace(async (workspace) => {
      const { catalog } = await publishUpstreamCatalog(workspace);
      writeEmptyForkPackage(catalog, "~> 1.4");
      process.chdir(catalog);
      assert.equal(await runCli(["install"]), 0);
      writeFileSync(
        join(catalog, "packages/fork-base/README.md"),
        "edited readme\n",
      );
      const chunks: string[] = [];
      const previousWrite = process.stdout.write.bind(process.stdout);
      process.stdout.write = ((chunk: string | Uint8Array) => {
        chunks.push(
          typeof chunk === "string"
            ? chunk
            : Buffer.from(chunk).toString("utf8"),
        );
        return true;
      }) as typeof process.stdout.write;
      try {
        assert.equal(await runCli(["outdated"]), 0);
      } finally {
        process.stdout.write = previousWrite;
      }
      const stdout = chunks.join("");
      assert.match(stdout, /README.md/);
      assert.match(stdout, /differs/);
    });
  });
});
