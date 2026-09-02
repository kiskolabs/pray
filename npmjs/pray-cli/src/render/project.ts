import { mkdirSync } from "node:fs";
import { resolve } from "node:path";
import { normalizeLineEndings } from "../hashing.js";
import type { Lockfile } from "../lockfile/types.js";
import { targetMode, targetScoped } from "../manifest/destination.js";
import type { ManifestTarget } from "../manifest/types.js";
import { validateDestinationPath } from "../manifest/validate.js";
import type { ResolvedProject } from "../resolve/types.js";
import { ensureHtmlCommentComposeDest } from "./compose-dest.js";
import { materializeProvisionedExports } from "./dest.js";
import { renderLegacyCompose } from "./legacy.js";
import { relocateManagedSpans } from "./patch.js";
import { ensureSafeDestinationAncestors } from "./path-guard.js";
import {
  layoutRenderedContent,
  writeRenderedContent,
} from "./rendered-dest.js";
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
  previousLockfile?: Lockfile,
): void {
  for (const target of rendered) {
    validateDestinationPath(target.path);
    ensureSafeDestinationAncestors(
      project.projectRoot,
      target.path,
      target.path,
    );
    const path = resolve(project.projectRoot, target.path);
    mkdirSync(resolve(path, ".."), { recursive: true });
    ensureSafeDestinationAncestors(
      project.projectRoot,
      target.path,
      target.path,
    );
    writeRenderedContent(path, target.path, target.content);
  }
  materializeProvisionedExports(project, previousLockfile);
}

export function layoutRenderedTargets(
  project: ResolvedProject,
  rendered: RenderedTarget[],
): RenderedTarget[] {
  return rendered.map((target) => {
    validateDestinationPath(target.path);
    ensureSafeDestinationAncestors(
      project.projectRoot,
      target.path,
      target.path,
    );
    const content = layoutRenderedContent(
      resolve(project.projectRoot, target.path),
      target.path,
      target.content,
    );
    return {
      ...target,
      content,
      managedSpans: relocateManagedSpans(content, target.managedSpans),
    };
  });
}

function renderTarget(
  project: ResolvedProject,
  target: ManifestTarget,
  output: string,
): RenderedTarget {
  ensureHtmlCommentComposeDest(output);
  if (targetScoped(target) && targetMode(target) === "compose") {
    return renderScopedCompose(project, target, output);
  }
  return renderLegacyCompose(project, target, output);
}

export { plannedProvisionedFiles } from "./provisioned.js";
export { normalizeLineEndings };
