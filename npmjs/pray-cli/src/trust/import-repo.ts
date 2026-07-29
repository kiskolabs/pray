import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { PrayError } from "../errors.js";
import { gitSourceCacheDirectory } from "../git/sources.js";
import {
  appendMissingKeys,
  mutableRuleForMatchPrefix,
  normalizeKey,
} from "./policy-helpers.js";
import { loadTrustPolicy, saveTrustPolicy } from "./store.js";

export function importSigningKeysFromRepository(
  projectRoot: string,
  sourceUrl: string,
  matchPrefix: string | undefined,
): number {
  const cloneUrl = sourceUrl.replace(/^git\+/, "");
  const repository = gitSourceCacheDirectory(projectRoot, cloneUrl);
  if (!existsSync(join(repository, ".git"))) {
    throw PrayError.resolution(
      `no cached git repository for ${cloneUrl} at ${repository}`,
    );
  }
  const keys = repositorySigningKeys(repository);
  if (keys.length === 0) {
    throw PrayError.unsupported(
      `no commit signing key/fingerprint found for HEAD in ${repository}`,
    );
  }
  const policy = loadTrustPolicy();
  const rule = mutableRuleForMatchPrefix(policy, matchPrefix ?? cloneUrl);
  const result = appendMissingKeys(rule.allowed_signing_keys, keys);
  rule.allowed_signing_keys = result.list;
  saveTrustPolicy(policy);
  return result.added;
}

function repositorySigningKeys(repository: string): string[] {
  const keys: string[] = [];
  for (const format of ["%GK", "%GF"] as const) {
    const value = gitLogFormat(repository, format);
    if (!value) {
      continue;
    }
    const normalized = normalizeKey(value);
    if (!keys.includes(normalized)) {
      keys.push(normalized);
    }
  }
  return keys;
}

function gitLogFormat(repository: string, format: string): string | undefined {
  const result = spawnSync("git", ["log", "-1", `--format=${format}`], {
    cwd: repository,
    encoding: "utf8",
  });
  if (result.status !== 0) {
    return undefined;
  }
  const value = (result.stdout ?? "").trim();
  return value.length > 0 ? value : undefined;
}
