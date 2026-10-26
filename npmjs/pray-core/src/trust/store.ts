import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";
import { parse, stringify } from "smol-toml";
import { PrayError } from "../errors.js";
import { parseTrustPolicyValue } from "./parse.js";
import type { TrustPolicy } from "./types.js";

export function trustHome(): string {
  return process.env.PRAY_HOME ?? join(homedir(), ".pray");
}

export function trustPolicyPath(): string {
  return join(trustHome(), "trust.toml");
}

export function defaultTrustPolicy(): TrustPolicy {
  return {
    default: { allow: true },
    rules: [],
  };
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
