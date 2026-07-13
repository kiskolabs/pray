import type { ManagedSpanRecord } from "../lockfile/types.js";

export interface RenderedTarget {
  path: string;
  content: string;
  managedSpans: ManagedSpanRecord[];
}
