import { basename } from "node:path";
import type { ResolvedProject } from "../resolve/types.js";
import type { ContentBuilder } from "./content-builder.js";

export function appendHeaderIfEnabled(
  builder: ContentBuilder,
  project: ResolvedProject,
  output: string,
): void {
  if (!project.manifest.render.header) {
    return;
  }
  const outputName = basename(output);
  builder.appendLine("<!-- pray:0 ignore-comments -->");
  builder.appendEmptyLine();
  builder.appendLine("# Agent context");
  builder.appendEmptyLine();
  builder.appendLine(
    `Do not edit managed blocks in \`${outputName}\` or provisioned files under \`.agents/\`.`,
  );
  builder.appendLine(
    "To change shared guidance, update `Prayfile` and run `pray install`.",
  );
  builder.appendEmptyLine();
}
