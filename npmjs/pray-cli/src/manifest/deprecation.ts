export const DEPRECATED_TARGET = "target";
export const DEPRECATED_OUTPUT = "output";
export const DEPRECATED_AGENT = "agent";

const REPLACEMENTS: Record<string, string> = {
  [DEPRECATED_TARGET]: "compose` / `tree",
  [DEPRECATED_OUTPUT]: "compose",
  [DEPRECATED_AGENT]: "pray",
};

export function noteDeprecatedKeyword(
  keywords: string[] | undefined,
  keyword: string,
): string[] {
  if (!(keyword in REPLACEMENTS)) {
    return keywords ?? [];
  }
  const current = keywords ?? [];
  if (current.includes(keyword)) {
    return current;
  }
  return [...current, keyword];
}

export function deprecationWarnings(keywords: string[] | undefined): string[] {
  return (keywords ?? [])
    .map((keyword) => {
      const replacement = REPLACEMENTS[keyword];
      if (!replacement) {
        return undefined;
      }
      return `warning: \`${keyword}\` is deprecated and will be removed in version 2; prefer \`${replacement}\``;
    })
    .filter((warning): warning is string => warning !== undefined);
}

export function emitDeprecationWarnings(keywords: string[] | undefined): void {
  for (const warning of deprecationWarnings(keywords)) {
    console.error(warning);
  }
}
