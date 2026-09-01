import { mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { normalizeLineEndings } from "../hashing.js";
import { targetMode, targetScoped } from "../manifest/destination.js";
import type { ManifestTarget } from "../manifest/types.js";
import { validateProjectRelativePath } from "../manifest/validate.js";
import type { ResolvedProject } from "../resolve/types.js";
import { renderLegacyCompose } from "./legacy.js";
import { materializeProvisionedExports } from "./provisioned.js";
import { renderScopedCompose } from "./scoped.js";
import type { RenderedTarget } from "./types.js";

export function renderProject(project: ResolvedProject): RenderedTarget[] {
  const rendered: RenderedTarget[] = [];
  for (const target of project.manifest.targets) {
    const output = target.outputs[0];
    if (!output) {
      continue;
    }
    rendered.push(renderTarget(project, target, output));
  }
  return rendered;
}

export function writeRenderedTargets(
  project: ResolvedProject,
  rendered: RenderedTarget[],
): void {
  for (const target of rendered) {
    validateProjectRelativePath(target.path);
    const path = resolve(project.projectRoot, target.path);
    mkdirSync(resolve(path, ".."), { recursive: true });
    writeFileSync(path, target.content, "utf8");
  }
  materializeProvisionedExports(project);
}

function renderTarget(
  project: ResolvedProject,
  target: ManifestTarget,
  output: string,
): RenderedTarget {
  if (targetScoped(target) && targetMode(target) === "compose") {
    return renderScopedCompose(project, target, output);
  }
  return renderLegacyCompose(project, target, output);
}

export { plannedProvisionedFiles } from "./provisioned.js";
export { normalizeLineEndings };
