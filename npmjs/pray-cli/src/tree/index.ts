import type { ResolvedPackage, ResolvedProject } from "../resolve/types.js";

export function renderDependencyTree(project: ResolvedProject): string[] {
  const packageMap = new Map(
    project.packages.map((packageEntry) => [
      packageEntry.declaration.name,
      packageEntry,
    ]),
  );
  const lines = ["Dependency tree"];
  for (const packageEntry of project.packages) {
    renderTreeNode(packageEntry, packageMap, 0, new Set(), lines);
  }
  return lines;
}

function renderTreeNode(
  packageEntry: ResolvedPackage,
  packageMap: Map<string, ResolvedPackage>,
  depth: number,
  ancestry: Set<string>,
  lines: string[],
): void {
  const indent = "  ".repeat(depth);
  lines.push(
    `${indent}${packageEntry.declaration.name} ${packageEntry.spec.version}`,
  );
  if (ancestry.has(packageEntry.declaration.name)) {
    lines.push(`${indent}  (cycle)`);
    return;
  }
  ancestry.add(packageEntry.declaration.name);
  for (const dependency of packageEntry.spec.dependencies) {
    const resolved = packageMap.get(dependency.name);
    if (resolved) {
      renderTreeNode(resolved, packageMap, depth + 1, ancestry, lines);
    } else {
      lines.push(`${indent}  ${dependency.name} (unresolved)`);
    }
  }
  ancestry.delete(packageEntry.declaration.name);
}

export function packageSourceSummary(packageEntry: ResolvedPackage): string {
  if (packageEntry.declaration.path) {
    return `path:${packageEntry.declaration.path}`;
  }
  if (packageEntry.declaration.source) {
    return packageEntry.declaration.source;
  }
  return "default";
}
