import { spawnSync } from "node:child_process";
import { PrayError } from "../errors.js";

export function runUpgradeCommand(): void {
  const result = spawnSync("npm", ["install", "-g", "pray-cli@latest"], {
    stdio: "inherit",
    env: process.env,
  });
  if (result.error) {
    throw PrayError.unsupported(
      `failed to run npm install: ${result.error.message} (is npm on PATH?)`,
    );
  }
  if (result.status !== 0) {
    throw PrayError.unsupported(
      "pray upgrade failed: npm install returned a non-zero exit status",
    );
  }
}
