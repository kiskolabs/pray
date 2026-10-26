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
      throw PrayError.usage(`unknown clean flag: ${argument}`);
    } else {
      throw PrayError.usage(`unexpected clean argument: ${argument}`);
    }
  }
  return { unused };
}
