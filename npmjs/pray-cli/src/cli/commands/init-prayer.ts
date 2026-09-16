import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { basename, join } from "node:path";
import { PrayError } from "../../errors.js";
import {
  DEFAULT_LOCAL_PRAYER_NAME,
  validateLocalPrayerName,
} from "../../local-prayer.js";
import { defaultManifestPath } from "../../lockfile/paths.js";
import { projectRoot } from "../invocation.js";
import {
  declareLocalPrayer,
  parsePrayerInitArguments,
  resolveLocalPrayerDirectory,
} from "./init-prayer-manifest.js";

export function runPrayerInit(argumentsList: string[] = []): void {
  const { name, directory } = parsePrayerInitArguments(argumentsList);
  if (existsSync(defaultManifestPath())) {
    scaffoldLocalPrayer(name, directory);
  } else {
    scaffoldStandalonePackage(name);
  }
}

function scaffoldLocalPrayer(name?: string, directory?: string): void {
  const packageName = validateLocalPrayerName(
    name && name.trim().length > 0 ? name : DEFAULT_LOCAL_PRAYER_NAME,
  );
  const source = resolveLocalPrayerDirectory(directory);
  const root = join(projectRoot(), source.directory, packageName);
  const prayspecPath = join(root, `${packageName}.prayspec`);
  if (existsSync(prayspecPath)) {
    throw PrayError.manifest(`package spec already exists: ${prayspecPath}`);
  }
  if (existsSync(root)) {
    throw PrayError.manifest(`prayer directory already exists: ${root}`);
  }
  mkdirSync(join(root, "exports"), { recursive: true });
  writeFileSync(prayspecPath, localPrayspec(source.name, packageName), "utf8");
  writeFileSync(join(root, "README.md"), `# ${packageName}\n`, "utf8");
  writeFileSync(
    join(root, "exports", `${packageName}.md`),
    `# ${packageName}\n`,
    "utf8",
  );
  declareLocalPrayer(`${source.name}/${packageName}`);
}

function scaffoldStandalonePackage(name?: string): void {
  const root = process.cwd();
  const packageName =
    name && name.trim().length > 0
      ? validateLocalPrayerName(name)
      : basename(root) || "prayer-package";
  const prayspecPath = join(root, `${packageName}.prayspec`);
  if (existsSync(prayspecPath)) {
    throw PrayError.manifest(`package spec already exists: ${prayspecPath}`);
  }
  writeFileSync(prayspecPath, standalonePrayspec(packageName), "utf8");
  const readmePath = join(root, "README.md");
  if (!existsSync(readmePath)) {
    writeFileSync(readmePath, `# ${packageName}\n`, "utf8");
  }
  mkdirSync(join(root, "exports"), { recursive: true });
}

function localPrayspec(sourceName: string, name: string): string {
  return `Package::Specification.new do |spec|
  spec.name = "${sourceName}/${name}"
  spec.summary = "Describe this package"
  spec.files = ["README.md", "exports/${name}.md"]
  spec.exports = {
    "${name}" => {
      type: "fragment",
      path: "exports/${name}.md"
    }
  }
end
`;
}

function standalonePrayspec(name: string): string {
  return `Package::Specification.new do |spec|
  spec.name = "${name}"
  spec.version = "0.1.0"
  spec.summary = "Describe this package"
  spec.files = ["README.md"]
  spec.exports = {}
end
`;
}
