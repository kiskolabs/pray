import {
  copyFileSync,
  existsSync,
  mkdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { basename, dirname, join } from "node:path";
import { findPrayspecFile } from "../package-spec/index.js";
import type { ResolvedProject } from "../resolve/types.js";

export function vendorProject(project: ResolvedProject): void {
  for (const packageEntry of project.packages) {
    const vendorRoot = join(
      project.projectRoot,
      ".pray",
      "vendor",
      packageEntry.declaration.name.replaceAll("/", "-"),
      packageEntry.spec.version,
    );
    mkdirSync(vendorRoot, { recursive: true });
    const prayspecPath = findPrayspecFile(packageEntry.root);
    copyFileSync(prayspecPath, join(vendorRoot, basename(prayspecPath)));
    writeFileSync(
      join(vendorRoot, "metadata.json"),
      JSON.stringify({
        name: packageEntry.spec.name,
        version: packageEntry.spec.version,
        tree_hash: packageEntry.treeHash,
        exports: packageEntry.selectedExports,
      }),
      "utf8",
    );
    for (const file of packageEntry.spec.files) {
      const destination = join(vendorRoot, file);
      mkdirSync(dirname(destination), { recursive: true });
      copyFileSync(join(packageEntry.root, file), destination);
    }
  }
}

export function cleanProjectCaches(projectRoot: string): void {
  for (const relative of [".pray/cache", ".pray/vendor", ".pray/state.json"]) {
    const path = join(projectRoot, relative);
    if (existsSync(path)) {
      rmSync(path, { recursive: true, force: true });
    }
  }
}
