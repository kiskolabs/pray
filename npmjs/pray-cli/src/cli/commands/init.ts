import { existsSync, writeFileSync } from "node:fs";
import { PrayError } from "../../errors.js";
import { defaultManifestPath } from "../../lockfile/paths.js";

export { runPrayerInit } from "./init-prayer.js";

export function runInit(argumentsList: string[]): void {
  const manifestPath = defaultManifestPath();
  if (existsSync(manifestPath)) {
    throw PrayError.manifest("Prayfile already exists");
  }
  const targetIndex = argumentsList.indexOf("--targets");
  const targetsArgument =
    targetIndex >= 0 ? argumentsList[targetIndex + 1] : undefined;
  const targets = targetsArgument ?? "tool_a";
  const targetNames = targets.split(",").map((name) => name.trim());
  const targetBlocks = targetNames
    .map(
      (name) => `target :${name} do
  output "INSTRUCTIONS.md"
  folder ".agents/skills"
end`,
    )
    .join("\n");
  const content = `prayfile "1"
${targetBlocks}
render mode: :managed,
  conflict: :fail,
  churn: :minimal
`;
  writeFileSync(manifestPath, content, "utf8");
  process.stdout.write(`created ${manifestPath}\n`);
}
