import { PrayError } from "../../errors.js";

export interface CleanOptions {
  unused: boolean;
}

export function parseCleanArguments(argumentsList: string[]): CleanOptions {
  let unused = false;
  for (const argument of argumentsList) {
    if (argument === "--unused" && !unused) {
      unused = true;
    } else if (argument.startsWith("--")) {
      throw PrayError.unsupported(`unknown clean flag: ${argument}`);
    } else {
      throw PrayError.unsupported(`unexpected clean argument: ${argument}`);
    }
  }
  return { unused };
}
