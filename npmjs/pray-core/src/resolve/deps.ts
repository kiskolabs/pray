import { PrayError } from "../errors.js";
import { findDependencyCycle } from "./dependency-graph.js";
import type { ResolvedPackage } from "./types.js";

export function rejectDependencyCycles(packages: ResolvedPackage[]): void {
  const edges = new Map<string, string[]>();
  for (const packageEntry of packages) {
    edges.set(
      packageEntry.declaration.name,
      packageEntry.spec.dependencies.map((dependency) => dependency.name),
    );
  }
  const cycle = findDependencyCycle(edges);
  if (!cycle) {
    return;
  }
  throw PrayError.resolution(
    `dependency cycle detected: ${cycle.join(" -> ")}`,
  );
}
