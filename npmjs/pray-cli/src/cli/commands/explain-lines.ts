import type { ResolvedPackage } from "pray-core/resolve/types.js";

export function packageDetailLines(packageEntry: ResolvedPackage): string[] {
  const required = packageEntry.spec.dependencies
    .filter((dependency) => !dependency.optional)
    .map((dependency) => dependency.name);
  const optional = packageEntry.spec.dependencies
    .filter((dependency) => dependency.optional)
    .map((dependency) => `${dependency.name} (${dependency.constraint})`);
  return [
    `exports: ${formatList(packageEntry.selectedExports)}`,
    `dependencies: ${formatList(required)}`,
    `optional referrals: ${formatList(optional)}`,
    `related prayers: ${formatList(packageEntry.spec.related)}`,
  ];
}

function formatList(values: string[]): string {
  return values.length === 0 ? "none" : values.join(", ");
}
