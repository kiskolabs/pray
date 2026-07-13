import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";
import { parse, stringify } from "smol-toml";
import { PrayError } from "../errors.js";

import { parseTrustPolicyValue } from "./parse.js";
import type { TrustPolicy } from "./types.js";

export type { TrustPolicy, TrustRule } from "./types.js";

export function trustHome(): string {
  return process.env.PRAY_HOME ?? join(homedir(), ".pray");
}

export function trustPolicyPath(): string {
  return join(trustHome(), "trust.toml");
}

export function loadTrustPolicy(): TrustPolicy {
  const path = trustPolicyPath();
  if (!existsSync(path)) {
    return defaultTrustPolicy();
  }
  try {
    return parseTrustPolicyValue(parse(readFileSync(path, "utf8")));
  } catch (error) {
    if (error instanceof PrayError) {
      throw error;
    }
    const message = error instanceof Error ? error.message : String(error);
    throw PrayError.parse("trust policy", message);
  }
}

export function saveTrustPolicy(policy: TrustPolicy): void {
  const path = trustPolicyPath();
  mkdirSync(join(path, ".."), { recursive: true });
  writeFileSync(path, `${stringify(policy)}\n`, "utf8");
}

export function defaultTrustPolicy(): TrustPolicy {
  return {
    default: { allow: true },
    rules: [],
  };
}

export function listTrustPolicy(): string {
  const policy = loadTrustPolicy();
  return stringify(policy);
}

export function addSigningKey(fingerprint: string): void {
  const policy = loadTrustPolicy();
  const keys = new Set(policy.default.allowed_signing_keys ?? []);
  keys.add(fingerprint);
  policy.default.allowed_signing_keys = [...keys].sort();
  saveTrustPolicy(policy);
}

export function removeSigningKey(fingerprint: string): void {
  const policy = loadTrustPolicy();
  policy.default.allowed_signing_keys = (policy.default.allowed_signing_keys ?? [])
    .filter((key) => key !== fingerprint);
  policy.rules = policy.rules.map((rule) => ({
    ...rule,
    allowed_signing_keys: rule.allowed_signing_keys?.filter(
      (key) => key !== fingerprint,
    ),
  }));
  saveTrustPolicy(policy);
}

export function setRequireSignedCommit(required: boolean): void {
  const policy = loadTrustPolicy();
  policy.default.require_signed_commit = required;
  saveTrustPolicy(policy);
}

export function setDefaultAllow(allow: boolean): void {
  const policy = loadTrustPolicy();
  policy.default.allow = allow;
  saveTrustPolicy(policy);
}

export function checkTrustPolicy(): string {
  return "trust check: no compromised key feed configured";
}

export function runTrustCommand(argumentsList: string[]): void {
  const [subcommand, ...rest] = argumentsList;
  switch (subcommand) {
    case "list":
      process.stdout.write(`${listTrustPolicy()}\n`);
      return;
    case "show":
      process.stdout.write(`${listTrustPolicy()}\n`);
      return;
    case "add-key": {
      const fingerprint = rest[0];
      if (!fingerprint) {
        throw PrayError.unsupported("trust add-key requires a fingerprint");
      }
      addSigningKey(fingerprint);
      return;
    }
    case "remove-key":
    case "revoke": {
      const fingerprint = rest[0];
      if (!fingerprint) {
        throw PrayError.unsupported("trust remove-key requires a fingerprint");
      }
      removeSigningKey(fingerprint);
      return;
    }
    case "set-signed": {
      const value = rest[0];
      setRequireSignedCommit(value === "true" || value === "1");
      return;
    }
    case "set-allow": {
      const value = rest[0];
      setDefaultAllow(value !== "false" && value !== "0");
      return;
    }
    case "check":
      process.stdout.write(`${checkTrustPolicy()}\n`);
      return;
    case "import-repo":
    case "import-registry":
      throw PrayError.unsupported(`${subcommand} is not implemented yet in pray-cli typescript`);
    default:
      throw PrayError.unsupported(`unknown trust subcommand: ${subcommand ?? "(none)"}`);
  }
}
