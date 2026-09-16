import type { Lockfile } from "../lockfile/types.js";
import { LOCAL_EMBED_PACKAGE } from "../lockfile/types.js";
import type { ResolvedProject } from "../resolve/types.js";

export function localSummaryLines(
  previous: Lockfile | undefined,
  project: ResolvedProject,
): string[] {
  const previousChecksums = previousLocalChecksums(previous);
  const lines: string[] = [];
  for (const local of project.localFiles) {
    if (local.content === "" && local.optional) {
      continue;
    }
    const checksum = local.sourceChecksum;
    const previousChecksum = previousChecksums.get(local.manifestPath);
    if (previousChecksum === undefined) {
      lines.push(`Installing ${local.manifestPath} (${checksum})`);
    } else if (previousChecksum === checksum) {
      lines.push(`Using ${local.manifestPath} (${checksum} checked)`);
    } else {
      lines.push(
        `Updating ${local.manifestPath} (${checksum} was ${previousChecksum})`,
      );
    }
  }
  return lines;
}

export function outdatedLocalLines(
  previous: Lockfile | undefined,
  project: ResolvedProject,
): string[] {
  const previousChecksums = previousLocalChecksums(previous);
  const lines: string[] = [];
  for (const local of project.localFiles) {
    if (local.content === "" && local.optional) {
      continue;
    }
    const checksum = local.sourceChecksum;
    const previousChecksum = previousChecksums.get(local.manifestPath);
    if (previousChecksum !== undefined && previousChecksum !== checksum) {
      lines.push(`${local.manifestPath} ${previousChecksum} -> ${checksum}`);
    } else if (previous !== undefined && previousChecksum === undefined) {
      lines.push(`${local.manifestPath} (new) -> ${checksum}`);
    }
  }
  return lines;
}

function previousLocalChecksums(
  previous: Lockfile | undefined,
): Map<string, string> {
  const checksums = new Map<string, string>();
  if (!previous) {
    return checksums;
  }
  for (const span of previous.managed_span) {
    if (span.package === LOCAL_EMBED_PACKAGE) {
      checksums.set(span.export, span.source_checksum);
    }
  }
  return checksums;
}
