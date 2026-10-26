import { AsyncLocalStorage } from "node:async_hooks";
export const activeTransaction = new AsyncLocalStorage<{
  root: string;
  replace(
    path: string,
    before: Buffer | undefined,
    after: Buffer | undefined,
  ): void;
}>();
export function replaceProjectFile(
  path: string,
  before: Buffer | undefined,
  after: Buffer | undefined,
): boolean {
  const journal = activeTransaction.getStore();
  if (!journal) return false;
  journal.replace(path, before, after);
  return true;
}
