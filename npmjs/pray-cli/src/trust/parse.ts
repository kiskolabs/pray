import { PrayError } from "../errors.js";
import {
  optionalString,
  requireBoolean,
  requireRecord,
  requireStringArray,
} from "../validation.js";
import type { TrustPolicy, TrustRule } from "./types.js";

const CONTEXT = "trust policy";

export function parseTrustPolicyValue(value: unknown): TrustPolicy {
  const record = requireRecord(value, CONTEXT);
  return {
    default: parseTrustRule(record.default ?? {}, "default"),
    rules: parseTrustRules(record.rules),
  };
}

function parseTrustRules(value: unknown): TrustRule[] {
  if (value === undefined) {
    return [];
  }
  if (!Array.isArray(value)) {
    throw PrayError.parse(CONTEXT, "rules must be an array");
  }
  return value.map((entry, index) =>
    parseTrustRule(entry, `rules[${index}]`),
  );
}

function parseTrustRule(value: unknown, context: string): TrustRule {
  const record = requireRecord(value, context);
  const rule: TrustRule = {};
  if (record.match_prefix !== undefined) {
    rule.match_prefix = optionalString(
      record.match_prefix,
      "match_prefix",
      context,
    );
  }
  if (record.allow !== undefined) {
    rule.allow = requireBoolean(record.allow, "allow", context);
  }
  if (record.require_signed_commit !== undefined) {
    rule.require_signed_commit = requireBoolean(
      record.require_signed_commit,
      "require_signed_commit",
      context,
    );
  }
  if (record.allowed_signing_keys !== undefined) {
    rule.allowed_signing_keys = requireStringArray(
      record.allowed_signing_keys,
      "allowed_signing_keys",
      context,
    );
  }
  if (record.allowed_host_keys !== undefined) {
    rule.allowed_host_keys = requireStringArray(
      record.allowed_host_keys,
      "allowed_host_keys",
      context,
    );
  }
  if (record.allowed_publishers !== undefined) {
    rule.allowed_publishers = requireStringArray(
      record.allowed_publishers,
      "allowed_publishers",
      context,
    );
  }
  return rule;
}
