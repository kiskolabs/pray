import { existsSync, readFileSync } from "node:fs";
import semver from "semver";
import { PrayError } from "../errors.js";
import { httpGetText, joinUrl } from "../http/client.js";
import {
  resolveDistributionPath,
  validateRegistryCacheIdentity,
} from "../sync/path-safety.js";
import { parseMetadata } from "./index.js";
import type {
  RegistryIndex,
  RegistryPackageMetadata,
  RegistryPackageVersion,
} from "./types.js";

export interface RegistrySearchHit {
  name: string;
  summary?: string;
}

export function searchRegistryIndexNames(
  index: RegistryIndex,
  query: string,
): string[] {
  validateRegistryIndex(index);
  const normalizedQuery = query.trim().toLowerCase();
  return index.packages
    .filter((name) => name.toLowerCase().includes(normalizedQuery))
    .sort();
}

export async function searchLocalRegistry(
  root: string,
  query: string,
  includeSummary = true,
): Promise<RegistrySearchHit[]> {
  const index = parseRegistryIndex(
    readFileSync(resolveDistributionPath(root, "v1/index.json"), "utf8"),
  );
  return searchRegistryIndexNames(index, query).map((name) => {
    if (!includeSummary) return { name };
    const metadataPath = resolveDistributionPath(
      root,
      `v1/packages/${name}.json`,
    );
    if (!existsSync(metadataPath)) return { name };
    return hitFromMetadata(name, readFileSync(metadataPath, "utf8"));
  });
}

export async function searchHttpRegistry(
  url: string,
  query: string,
  includeSummary = true,
): Promise<RegistrySearchHit[]> {
  const index = parseRegistryIndex(
    await httpGetText(joinUrl(url, "v1/index.json")),
  );
  const hits: RegistrySearchHit[] = [];
  for (const name of searchRegistryIndexNames(index, query)) {
    if (!includeSummary) {
      hits.push({ name });
      continue;
    }
    try {
      hits.push(
        hitFromMetadata(
          name,
          await httpGetText(joinUrl(url, `v1/packages/${name}.json`)),
        ),
      );
    } catch (error) {
      if (error instanceof PrayError && error.kind === "resolution") {
        hits.push({ name });
        continue;
      }
      throw error;
    }
  }
  return hits;
}

function parseRegistryIndex(text: string): RegistryIndex {
  let data: unknown;
  try {
    data = JSON.parse(text) as unknown;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw PrayError.integrity(`invalid distribution index: ${message}`);
  }
  if (!isRecord(data) || !Array.isArray(data.packages)) {
    throw PrayError.integrity("invalid distribution index");
  }
  const index = {
    spec: typeof data.spec === "string" ? data.spec : "",
    packages: data.packages,
  } as RegistryIndex;
  validateRegistryIndex(index);
  return index;
}

function validateRegistryIndex(index: RegistryIndex): void {
  if (
    index.spec !== "prayfile-distribution-1" ||
    !index.packages.every((name) => typeof name === "string")
  ) {
    throw PrayError.integrity("invalid distribution index");
  }
  for (const name of index.packages) {
    validateRegistryCacheIdentity(name, "0");
  }
}

function hitFromMetadata(name: string, text: string): RegistrySearchHit {
  const metadata = parseMetadata(text);
  if (metadata.name !== name) {
    throw PrayError.integrity(
      `registry metadata name ${metadata.name} does not match ${name}`,
    );
  }
  const summary = latestNonYankedSummary(metadata);
  return summary === undefined ? { name } : { name, summary };
}

function latestNonYankedSummary(
  metadata: RegistryPackageMetadata,
): string | undefined {
  const latest = metadata.versions
    .filter((version) => !version.yanked)
    .reduce<RegistryPackageVersion | undefined>((current, candidate) => {
      if (
        current === undefined ||
        semver.gt(candidate.version, current.version)
      ) {
        return candidate;
      }
      return current;
    }, undefined);
  const summary = latest?.derivedMetadata?.summary?.trim();
  return summary === "" ? undefined : summary;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
