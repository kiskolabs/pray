/**
 * Cross-runtime smoke for the full library entry (Node / Deno / Bun).
 * Run after `npm run build` from npmjs/pray-core.
 */
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const packageRoot = join(root, "..");
const distIndex = pathToFileURL(join(packageRoot, "dist/index.js")).href;
const { parseManifest, parseLockfile, manifestHash, lockfileHash } =
  await import(distIndex);

const manifest = parseManifest(`
prayfile "1"
source "default", "https://agents.example.com"
target :tool_a do
  output "INSTRUCTIONS.md"
end
`);

const lockText = readFileSync(
  join(packageRoot, "../../examples/simple-project/Prayfile.lock"),
  "utf8",
);
const lockfile = parseLockfile(lockText);

if (manifest.prayfileVersion !== "1") {
  throw new Error(`unexpected prayfile version: ${manifest.prayfileVersion}`);
}
if (!manifestHash(manifest).startsWith("sha256:")) {
  throw new Error("manifestHash missing sha256 prefix");
}
if (!lockfileHash(lockfile).startsWith("sha256:")) {
  throw new Error("lockfileHash missing sha256 prefix");
}
if ((lockfile.package?.length ?? 0) < 1) {
  throw new Error("expected fixture lockfile packages");
}

const runtime =
  typeof Bun !== "undefined"
    ? "bun"
    : typeof Deno !== "undefined"
      ? "deno"
      : "node";
console.log(`runtime-smoke ok (${runtime})`);
