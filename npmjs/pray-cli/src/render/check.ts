import { resolve } from "node:path";
import { PrayError } from "../errors.js";
import type { ResolvedProject } from "../resolve/types.js";
import { readRegularBytes } from "./destination-io.js";
import { layoutRenderedTargets } from "./project.js";
import type { RenderedTarget } from "./types.js";

export function ensureRenderedTargetsCurrent(
  project: ResolvedProject,
  rendered: RenderedTarget[],
): void {
  for (const target of layoutRenderedTargets(project, rendered)) {
    const path = resolve(project.projectRoot, target.path);
    const onDisk = readRegularBytes(path, target.path).toString("utf8");
    if (onDisk !== target.content) {
      throw PrayError.render(
        `${path} is stale; rerun pray install to regenerate it or pray plan to inspect the diff`,
      );
    }
  }
}
