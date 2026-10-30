import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { PrayError } from "../errors.js";
import type { Lockfile } from "../lockfile/types.js";
import { validateDestinationPath } from "../manifest/validate.js";
import type { ResolvedProject } from "../resolve/types.js";
import { readRegularBytes } from "./destination-io.js";
import { managedSpanChecksums } from "./patch.js";
import { ensureSafeDestinationAncestors } from "./path-guard.js";
import type { RenderedTarget } from "./types.js";

export function rejectManagedSpanConflicts(
  project: ResolvedProject,
  rendered: RenderedTarget[],
  previousLockfile: Lockfile,
): void {
  const activeByTarget = new Map(
    rendered.map((target) => [
      target.path,
      new Set(target.managedSpans.map((span) => span.id)),
    ]),
  );
  const checksumsByTarget = new Map<string, ReadonlyMap<string, string>>();
  for (const span of previousLockfile.managed_span) {
    if (!activeByTarget.get(span.target)?.has(span.id)) continue;
    let checksums = checksumsByTarget.get(span.target);
    if (checksums === undefined) {
      validateDestinationPath(span.target);
      ensureSafeDestinationAncestors(
        project.projectRoot,
        span.target,
        span.target,
      );
      const path = resolve(project.projectRoot, span.target);
      if (!existsSync(path)) continue;
      checksums = managedSpanChecksums(
        readRegularBytes(path, span.target).toString("utf8"),
      );
      checksumsByTarget.set(span.target, checksums);
    }
    const actual = checksums.get(span.id);
    if (actual !== undefined && actual !== span.ideal_checksum) {
      throw PrayError.render(
        `conflict: managed span \`${span.id}\` in \`${span.target}\` was edited (\`${span.package}::${span.export}\`)`,
      );
    }
  }
}
