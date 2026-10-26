import { PrayError } from "./errors.js";

const PLACEHOLDER_PREFIX = "((pray:";
const PLACEHOLDER_SUFFIX = "))";

export function isPraySymbolKey(key: string): boolean {
  if (key.length === 0) {
    return false;
  }
  return /^[A-Za-z0-9._/-]+$/.test(key);
}

export function substitutePraySymbols(
  text: string,
  symbols: Record<string, string> | Map<string, string>,
): string {
  const lookup =
    symbols instanceof Map ? symbols : new Map(Object.entries(symbols));
  let output = "";
  let rest = text;

  while (true) {
    const start = rest.indexOf(PLACEHOLDER_PREFIX);
    if (start < 0) {
      output += rest;
      return output;
    }
    output += rest.slice(0, start);
    const afterPrefix = rest.slice(start + PLACEHOLDER_PREFIX.length);
    const end = afterPrefix.indexOf(PLACEHOLDER_SUFFIX);
    if (end < 0) {
      throw PrayError.render("unclosed ((pray:...) placeholder");
    }
    const path = afterPrefix.slice(0, end);
    if (!isPraySymbolKey(path)) {
      throw PrayError.render(`invalid ((pray:...)) path \`${path}\``);
    }
    const value = lookup.get(path);
    if (value === undefined) {
      throw PrayError.render(
        `unknown pray symbol \`${path}\`; declare it in \`pray do ... end\``,
      );
    }
    output += value;
    rest = afterPrefix.slice(end + PLACEHOLDER_SUFFIX.length);
  }
}
