import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { PrayError } from "../errors.js";
import { httpGet, joinUrl } from "../http/client.js";
import {
  appendMissingKeys,
  mutableRuleForMatchPrefix,
  normalizeKey,
} from "./policy-helpers.js";
import { loadTrustPolicy, saveTrustPolicy } from "./store.js";

export interface ImportRegistryResult {
  publishersAdded: number;
  hostKeysAdded: number;
}

interface SshPublishersConfig {
  publishers?: Array<{ fingerprint?: string }>;
}

export async function importRegistryTrust(
  sourceUrl: string,
  matchPrefix: string | undefined,
  includeHostKey: boolean,
): Promise<ImportRegistryResult> {
  const prefix = matchPrefix ?? sourceUrl;
  const config = await fetchSshPublishers(sourceUrl);
  if (!config) {
    throw PrayError.unsupported(
      `no v1/ssh_publishers.json found for ${sourceUrl}`,
    );
  }
  const publisherFingerprints = (config.publishers ?? [])
    .map((entry) => normalizeKey(entry.fingerprint ?? ""))
    .filter((fingerprint) => fingerprint.length > 0);
  if (publisherFingerprints.length === 0) {
    throw PrayError.unsupported(
      `v1/ssh_publishers.json for ${sourceUrl} lists no publisher fingerprints`,
    );
  }

  if (
    includeHostKey &&
    (sourceUrl.startsWith("pray+ssh://") || sourceUrl.startsWith("ssh+pray://"))
  ) {
    throw PrayError.unsupported(
      "host-key import for pray+ssh:// is not implemented yet in pray-cli typescript",
    );
  }

  const policy = loadTrustPolicy();
  const rule = mutableRuleForMatchPrefix(policy, prefix);
  const publishers = appendMissingKeys(
    rule.allowed_publishers,
    publisherFingerprints,
  );
  rule.allowed_publishers = publishers.list;
  saveTrustPolicy(policy);
  return { publishersAdded: publishers.added, hostKeysAdded: 0 };
}

async function fetchSshPublishers(
  sourceUrl: string,
): Promise<SshPublishersConfig | undefined> {
  const localRoot = localDistributionRoot(sourceUrl);
  if (localRoot) {
    const path = join(localRoot, "v1", "ssh_publishers.json");
    if (!existsSync(path)) {
      return undefined;
    }
    return JSON.parse(readFileSync(path, "utf8")) as SshPublishersConfig;
  }
  if (
    sourceUrl.startsWith("pray+ssh://") ||
    sourceUrl.startsWith("ssh+pray://")
  ) {
    throw PrayError.unsupported(
      "pray+ssh:// registry import is not implemented yet in pray-cli typescript",
    );
  }
  if (sourceUrl.startsWith("http://") || sourceUrl.startsWith("https://")) {
    try {
      const bytes = await httpGet(joinUrl(sourceUrl, "v1/ssh_publishers.json"));
      return JSON.parse(bytes.toString("utf8")) as SshPublishersConfig;
    } catch (error) {
      if (
        error instanceof PrayError &&
        error.kind === "resolution" &&
        error.message.includes("404")
      ) {
        return undefined;
      }
      throw error;
    }
  }
  throw PrayError.unsupported(
    `unsupported registry source for import: ${sourceUrl}`,
  );
}

function localDistributionRoot(sourceUrl: string): string | undefined {
  const path = sourceUrl.startsWith("file://")
    ? sourceUrl.slice("file://".length)
    : sourceUrl;
  return existsSync(path) ? path : undefined;
}
