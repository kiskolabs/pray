import type { Lockfile } from "../lockfile/types.js";
import type { RenderedTarget } from "./types.js";

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
