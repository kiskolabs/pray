import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

export function writePackage(
  root: string,
  directory: string,
  packageName: string,
  exportName: string,
  exportKind: string,
  exportPath: string,
  body: string | Uint8Array,
): void {
  const packageRoot = join(root, "packages", directory);
  mkdirSync(join(packageRoot, "exports"), { recursive: true });
  writeFileSync(
    join(packageRoot, `${directory}.prayspec`),
    `
Package::Specification.new do |spec|
  spec.name = "${packageName}"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = ["${exportPath}"]
  spec.exports = {
    "${exportName}" => {
      type: "${exportKind}",
      path: "${exportPath}",
      summary: "${exportName}"
    }
  }
end
`,
  );
  writeFileSync(join(packageRoot, exportPath), body);
}
