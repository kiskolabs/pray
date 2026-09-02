export const DEPRECATED_TARGET = "target";
export const DEPRECATED_OUTPUT = "output";
export const DEPRECATED_AGENT = "agent";
export const DEPRECATED_SKILLS = "skills";
export const DEPRECATED_SKILL = "skill";
export const DEPRECATED_SPEC_SKILLS = "spec.skills";

const REPLACEMENTS: Record<string, string> = {
  [DEPRECATED_TARGET]: "compose` / `tree",
  [DEPRECATED_OUTPUT]: "compose",
  [DEPRECATED_AGENT]: "pray",
  [DEPRECATED_SKILLS]: "tree` / `folder",
  [DEPRECATED_SKILL]: "folder",
  [DEPRECATED_SPEC_SKILLS]: "a folder export",
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

export function emitResolvedDeprecationWarnings(
  keywords: string[] | undefined,
  packages: Array<{
    spec: {
      skills: Map<string, unknown>;
      exports: Map<string, { kind: string }>;
    };
  }>,
): void {
  emitDeprecationWarnings(keywords);
  const seen = new Set<string>();
  for (const resolved of packages) {
    for (const warning of packageSpecDeprecationWarnings(resolved.spec)) {
      if (seen.has(warning)) {
        continue;
      }
      seen.add(warning);
      console.error(warning);
    }
  }
}

export function packageSpecDeprecationWarnings(spec: {
  skills: Map<string, unknown>;
  exports: Map<string, { kind: string }>;
}): string[] {
  const keywords: string[] = [];
  if (spec.skills.size > 0) {
    keywords.push(DEPRECATED_SPEC_SKILLS);
  }
  for (const exportEntry of spec.exports.values()) {
    if (exportEntry.kind === DEPRECATED_SKILL) {
      keywords.push(DEPRECATED_SKILL);
      break;
    }
  }
  return deprecationWarnings(keywords);
}
