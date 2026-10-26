/**
 * Cross-runtime smoke for the full CLI entry (Node / Deno / Bun).
 * Run after `npm run build` from npmjs/pray-cli.
 */
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const packageRoot = join(dirname(fileURLToPath(import.meta.url)), "..");
const distCli = pathToFileURL(join(packageRoot, "dist/cli/main.js")).href;
const { runCli } = await import(distCli);

const versionCode = await runCli(["-q", "version"]);
if (versionCode !== 0) {
  throw new Error(`runCli(["version"]) exited ${versionCode}`);
}

const runtime =
  typeof Bun !== "undefined"
    ? "bun"
    : typeof Deno !== "undefined"
      ? "deno"
      : "node";
console.log(`pray-cli runtime-smoke ok (${runtime})`);
