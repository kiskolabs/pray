import type { ManifestTarget } from "../manifest/types.js";
import type { ResolvedProject } from "../resolve/types.js";
import { composeHeaderText } from "./compose-dest.js";
import type { ContentBuilder } from "./content-builder.js";

export function appendHeaderIfEnabled(
  builder: ContentBuilder,
  project: ResolvedProject,
  target: ManifestTarget,
  output: string,
): void {
  const header = composeHeaderText(
    target,
    output,
    project.manifest.render.header,
  );
  if (!header) {
    return;
  }
  builder.appendBody(header);
  builder.appendEmptyLine();
}
