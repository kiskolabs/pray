import { basename, extname } from "node:path";
import { PrayError } from "../errors.js";
import type { ManifestTarget } from "../manifest/types.js";

const HTML_COMMENT_EXTENSIONS = new Set([".md", ".markdown", ".html", ".htm"]);
const BINARY_EXTENSIONS = new Set([
  ".png",
  ".jpg",
  ".jpeg",
  ".gif",
  ".webp",
  ".ico",
  ".pdf",
  ".zip",
  ".gz",
  ".tgz",
  ".tar",
  ".wasm",
  ".bin",
  ".woff",
  ".woff2",
  ".exe",
  ".dylib",
  ".so",
]);

export function composeWritesHeader(
  target: ManifestTarget,
  output: string,
  projectHeader: boolean,
): boolean {
  if (target.header !== undefined) {
    return target.header;
  }
  return projectHeader && isAgentsMarkdown(output);
}

export function composeHeaderText(
  target: ManifestTarget,
  output: string,
  projectHeader: boolean,
): string | undefined {
  if (!composeWritesHeader(target, output, projectHeader)) {
    return undefined;
  }
  const name = basename(output);
  const guidance = isAgentsMarkdown(output)
    ? `Do not edit managed blocks in \`${name}\` or provisioned files under \`.agents/\`.`
    : `Do not edit managed blocks in \`${name}\`.`;
  return `<!-- pray:0 ignore-comments -->\n\n# Agent context\n\n${guidance}\nTo change shared guidance, update \`Prayfile\` and run \`pray install\`.`;
}

export function ensureHtmlCommentComposeDest(output: string): void {
  const dest = output.replaceAll("\\", "/");
  const extension = extname(output).toLowerCase();
  if (HTML_COMMENT_EXTENSIONS.has(extension)) {
    return;
  }
  if (extension === ".json") {
    throw PrayError.render(
      `compose cannot write JSON; use file: "${dest}" for unmarked bytes`,
    );
  }
  if (BINARY_EXTENSIONS.has(extension)) {
    throw PrayError.render(
      `compose cannot write a binary file; use file: "${dest}" for unmarked bytes`,
    );
  }
  throw PrayError.render(
    `compose cannot write this file type; use file: "${dest}" for unmarked bytes`,
  );
}

function isAgentsMarkdown(output: string): boolean {
  return basename(output) === "AGENTS.md";
}
