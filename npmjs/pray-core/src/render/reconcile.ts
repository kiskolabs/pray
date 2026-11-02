import type { Lockfile } from "../lockfile/types.js";
import { patchRenderedContent } from "./patch.js";
import type { RenderedTarget } from "./types.js";

export function reconcileRenderedContent(
  existing: string,
  fresh: string,
  retired: ReadonlyMap<string, string> = new Map(),
): string {
  return patchRenderedContent(existing, fresh, retired);
}

export function retiredSpanChecksums(
  previousLockfile: Lockfile | undefined,
  target: RenderedTarget,
): ReadonlyMap<string, string> {
  const currentIds = new Set(target.managedSpans.map((span) => span.id));
  return new Map(
    (previousLockfile?.managed_span ?? [])
      .filter((span) => span.target === target.path && !currentIds.has(span.id))
      .map((span) => [span.id, span.ideal_checksum]),
  );
}
