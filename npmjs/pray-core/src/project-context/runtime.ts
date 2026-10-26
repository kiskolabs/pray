import type { ProjectInvocationContext } from "./types.js";

let activeContext: ProjectInvocationContext | undefined;

export function setActiveInvocationContext(
  context: ProjectInvocationContext | undefined,
): void {
  activeContext = context;
}

export function activeInvocationContext():
  | ProjectInvocationContext
  | undefined {
  return activeContext;
}
