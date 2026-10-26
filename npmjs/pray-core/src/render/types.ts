import type { ManagedSpanRecord } from "../lockfile/types.js";

/** In-memory render result for one destination path. */
export interface RenderedTarget {
  /** Destination path relative to the project root. */
  path: string;
  /** Full file content to write. */
  content: string;
  /** Managed spans present in the content. */
  managedSpans: ManagedSpanRecord[];
}
