/**
 * Cloudflare Workers parse-surface smoke: browser-platform bundle must load
 * without Node builtins, then parseManifest must run.
 */
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const entry = join(packageRoot, "src/parse.ts");
const outDir = mkdtempSync(join(tmpdir(), "pray-workers-parse-"));
const outfile = join(outDir, "parse.mjs");

try {
  const bundled = spawnSync(
    "npx",
    [
      "--yes",
      "esbuild",
      entry,
      "--bundle",
      "--platform=browser",
      "--format=esm",
      `--outfile=${outfile}`,
    ],
    { cwd: packageRoot, encoding: "utf8" },
  );
  if (bundled.status !== 0) {
    throw new Error(
      `esbuild browser bundle failed:\n${bundled.stderr || bundled.stdout}`,
    );
  }

  const { parseManifest, manifestHash, parseLockfile, serializeLockfile } =
    await import(pathToFileURL(outfile).href);

  const manifest = parseManifest(`prayfile "1"\n`);
  const lockfile = parseLockfile(`
prayfile_lock = "1"
spec = "0.1"
generated_by = "pray workers-smoke"
manifest_hash = "sha256:abc"
source = []
package = []
target = []
managed_span = []
`);

  if (manifest.prayfileVersion !== "1") {
    throw new Error("parseManifest failed in workers bundle");
  }
  if (!manifestHash(manifest).startsWith("sha256:")) {
    throw new Error("manifestHash failed in workers bundle");
  }
  if (!serializeLockfile(lockfile).includes("prayfile_lock")) {
    throw new Error("serializeLockfile failed in workers bundle");
  }

  console.log("workers-parse-smoke ok");
} finally {
  rmSync(outDir, { recursive: true, force: true });
}
