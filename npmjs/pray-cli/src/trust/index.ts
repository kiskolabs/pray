import { stringify } from "smol-toml";
import { PrayError } from "../errors.js";
import { gitSourceCacheDirectory } from "../git/sources.js";
import { importRegistryTrust } from "./import-registry.js";
import { importSigningKeysFromRepository } from "./import-repo.js";
import {
  defaultTrustPolicy,
  loadTrustPolicy,
  saveTrustPolicy,
  trustHome,
  trustPolicyPath,
} from "./store.js";

export type { TrustPolicy, TrustRule } from "./types.js";
export {
  defaultTrustPolicy,
  loadTrustPolicy,
  saveTrustPolicy,
  trustHome,
  trustPolicyPath,
};

export function listTrustPolicy(): string {
  return stringify(loadTrustPolicy());
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
  policy.default.allowed_signing_keys = (
    policy.default.allowed_signing_keys ?? []
  ).filter((key) => key !== fingerprint);
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

export async function runTrustCommand(
  argumentsList: string[],
  projectRoot: string,
): Promise<void> {
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
    case "import-repo": {
      const { sourceUrl, matchPrefix } = parseImportRepoArguments(rest);
      const added = importSigningKeysFromRepository(
        projectRoot,
        sourceUrl,
        matchPrefix,
      );
      const cloneUrl = sourceUrl.replace(/^git\+/, "");
      process.stdout.write(
        `imported ${added} key(s) from ${gitSourceCacheDirectory(projectRoot, cloneUrl)}\n`,
      );
      return;
    }
    case "import-registry": {
      const options = parseImportRegistryArguments(rest);
      const result = await importRegistryTrust(
        options.sourceUrl,
        options.matchPrefix,
        options.includeHostKey,
      );
      process.stdout.write(
        `imported ${result.publishersAdded} publisher fingerprint(s) and ${result.hostKeysAdded} host key(s) for ${options.matchPrefix ?? options.sourceUrl}\n`,
      );
      return;
    }
    default:
      throw PrayError.unsupported(
        `unknown trust subcommand: ${subcommand ?? "(none)"}`,
      );
  }
}

function parseImportRepoArguments(argumentsList: string[]): {
  sourceUrl: string;
  matchPrefix?: string;
} {
  const sourceUrl = argumentsList[0];
  if (!sourceUrl) {
    throw PrayError.unsupported("trust import-repo requires SOURCE_URL");
  }
  let matchPrefix: string | undefined;
  for (let index = 1; index < argumentsList.length; index += 1) {
    const argument = argumentsList[index]!;
    if (argument === "--match-prefix") {
      const value = argumentsList[index + 1];
      if (!value) {
        throw PrayError.unsupported("--match-prefix requires VALUE");
      }
      matchPrefix = value;
      index += 1;
      continue;
    }
    throw PrayError.unsupported(
      `unknown trust import-repo argument: ${argument}`,
    );
  }
  return { sourceUrl, matchPrefix };
}

function parseImportRegistryArguments(argumentsList: string[]): {
  sourceUrl: string;
  matchPrefix?: string;
  includeHostKey: boolean;
} {
  const sourceUrl = argumentsList[0];
  if (!sourceUrl) {
    throw PrayError.unsupported("trust import-registry requires SOURCE_URL");
  }
  let matchPrefix: string | undefined;
  let includeHostKey =
    sourceUrl.startsWith("pray+ssh://") || sourceUrl.startsWith("ssh+pray://");
  for (let index = 1; index < argumentsList.length; index += 1) {
    const argument = argumentsList[index]!;
    switch (argument) {
      case "--match-prefix": {
        const value = argumentsList[index + 1];
        if (!value) {
          throw PrayError.unsupported("--match-prefix requires VALUE");
        }
        matchPrefix = value;
        index += 1;
        break;
      }
      case "--host-key":
        includeHostKey = true;
        break;
      case "--no-host-key":
        includeHostKey = false;
        break;
      default:
        throw PrayError.unsupported(
          `unknown trust import-registry argument: ${argument}`,
        );
    }
  }
  return { sourceUrl, matchPrefix, includeHostKey };
}
