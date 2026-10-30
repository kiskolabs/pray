import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { runCli } from "./cli/main.js";

describe("pray search", () => {
  it("finds undeclared packages in a local distribution root", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-cli-search-"));
    try {
      mkdirSync(join(root, "v1/packages/sample"), { recursive: true });
      writeFileSync(
        join(root, "v1/index.json"),
        JSON.stringify({
          spec: "prayfile-distribution-1",
          packages: ["sample/base"],
        }),
      );
      writeFileSync(
        join(root, "v1/packages/sample/base.json"),
        JSON.stringify({
          name: "sample/base",
          versions: [
            {
              version: "1.0.0",
              artifact: "v1/artifacts/sample/base/1.0.0/package.praypkg",
              yanked: false,
              derived_metadata: { summary: "shared guidance" },
            },
          ],
        }),
      );
      const output: string[] = [];
      const previousWrite = process.stdout.write.bind(process.stdout);
      process.stdout.write = ((chunk: string | Uint8Array) => {
        output.push(String(chunk));
        return true;
      }) as typeof process.stdout.write;
      try {
        assert.equal(await runCli(["search", "base", "--root", root]), 0);
      } finally {
        process.stdout.write = previousWrite;
      }
      assert.equal(output.join(""), "sample/base — shared guidance\n");
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it("rejects a missing selector value", async () => {
    assert.equal(await runCli(["search", "base", "--root"]), 2);
  });
});
