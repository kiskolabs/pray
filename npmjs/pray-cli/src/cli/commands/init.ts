import { existsSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { PrayError } from "../../errors.js";
import { defaultManifestPath } from "../../lockfile/paths.js";

export function runInit(argumentsList: string[]): void {
  const manifestPath = defaultManifestPath();
  if (existsSync(manifestPath)) {
    throw PrayError.manifest("Prayfile already exists");
  }
  const targetIndex = argumentsList.indexOf("--targets");
  const targetsArgument = targetIndex >= 0 ? argumentsList[targetIndex + 1] : undefined;
  const targets = targetsArgument ?? "tool_a";
  const targetNames = targets.split(",").map((name) => name.trim());
  const targetBlocks = targetNames
    .map(
      (name) => `target :${name} do
  output "INSTRUCTIONS.md"
  skills ".agents/skills"
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

export function runPrayerInit(): void {
  const root = process.cwd();
  const packageName = root.split("/").pop() || "prayer-package";
  const prayspecPath = join(root, `${packageName}.prayspec`);
  if (existsSync(prayspecPath)) {
    throw PrayError.manifest(`package spec already exists: ${prayspecPath}`);
  }
  writeFileSync(
    prayspecPath,
    `Package::Specification.new do |spec|
  spec.name = "${packageName}"
  spec.version = "0.1.0"
  spec.summary = "Prayer package"
  spec.files = []
end
`,
    "utf8",
  );
}
