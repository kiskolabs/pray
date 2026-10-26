import type { TrustPolicy, TrustRule } from "./types.js";

export function normalizeKey(value: string): string {
  return value.trim().toUpperCase();
}

export function mutableRuleForMatchPrefix(
  policy: TrustPolicy,
  matchPrefix: string,
): TrustRule {
  const existing = policy.rules.find(
    (rule) => rule.match_prefix === matchPrefix,
  );
  if (existing) {
    return existing;
  }
  const rule: TrustRule = {
    match_prefix: matchPrefix,
    allow: true,
    require_signed_commit: false,
    allowed_signing_keys: [],
    allowed_host_keys: [],
    allowed_publishers: [],
  };
  policy.rules.push(rule);
  return rule;
}

export function appendMissingKeys(
  target: string[] | undefined,
  keys: string[],
): { list: string[]; added: number } {
  const list = [...(target ?? [])];
  let added = 0;
  for (const key of keys) {
    const normalized = normalizeKey(key);
    if (!normalized) {
      continue;
    }
    if (list.some((existing) => normalizeKey(existing) === normalized)) {
      continue;
    }
    list.push(normalized);
    added += 1;
  }
  return { list, added };
}
