import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { PrayError } from "./errors.js";

export const DISTRIBUTION_CONFIG_SPEC = "pray-distribution-config-1";
const TORRENT_PROTOCOL = "torrent";
const DISTRIBUTION_CONFIG_FIELDS = new Set([
  "spec",
  "protocols",
  "bootstrap_trackers",
  "enable_dht",
]);

export interface RegistryDistributionSettings {
  spec: string;
  protocols: string[];
  bootstrapTrackers: string[];
  enableDht: boolean;
}

export function defaultDistributionSettings(): RegistryDistributionSettings {
  return {
    spec: DISTRIBUTION_CONFIG_SPEC,
    protocols: [],
    bootstrapTrackers: [],
    enableDht: false,
  };
}

export function allowsTorrent(settings: RegistryDistributionSettings): boolean {
  return settings.protocols.includes(TORRENT_PROTOCOL);
}

export function parseDistributionSettings(
  text: string,
): RegistryDistributionSettings {
  let data: Record<string, unknown>;
  try {
    data = JSON.parse(text) as Record<string, unknown>;
  } catch (error) {
    throw PrayError.parse(
      "distribution config",
      error instanceof Error ? error.message : String(error),
    );
  }
  if (data === null || typeof data !== "object" || Array.isArray(data)) {
    throw PrayError.parse("distribution config", "expected an object");
  }
  for (const key of Object.keys(data)) {
    if (!DISTRIBUTION_CONFIG_FIELDS.has(key)) {
      throw PrayError.parse("distribution config", `unsupported field: ${key}`);
    }
  }
  const settings: RegistryDistributionSettings = {
    spec: typeof data.spec === "string" ? data.spec : DISTRIBUTION_CONFIG_SPEC,
    protocols: Array.isArray(data.protocols)
      ? data.protocols.map((name) => String(name))
      : [],
    bootstrapTrackers: Array.isArray(data.bootstrap_trackers)
      ? data.bootstrap_trackers.map((name) => String(name))
      : [],
    enableDht: data.enable_dht === true,
  };
  validateDistributionSettings(settings);
  return settings;
}

export function readDistributionSettings(
  root: string,
): RegistryDistributionSettings {
  const path = join(root, "v1", "distribution.json");
  if (!existsSync(path)) {
    return defaultDistributionSettings();
  }
  return parseDistributionSettings(readFileSync(path, "utf8"));
}

export function writeDistributionSettings(
  root: string,
  settings: RegistryDistributionSettings = defaultDistributionSettings(),
): void {
  validateDistributionSettings(settings);
  const path = join(root, "v1", "distribution.json");
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(
    path,
    `${JSON.stringify(
      { spec: settings.spec, protocols: settings.protocols },
      null,
      2,
    )}\n`,
  );
}

function validateDistributionSettings(
  settings: RegistryDistributionSettings,
): void {
  if (settings.spec !== DISTRIBUTION_CONFIG_SPEC) {
    throw PrayError.parse(
      "distribution config",
      `unsupported distribution config spec: ${settings.spec}`,
    );
  }
  for (const name of settings.protocols) {
    if (name !== TORRENT_PROTOCOL) {
      throw PrayError.parse(
        "distribution config",
        `unsupported protocol: ${name}`,
      );
    }
  }
  if (settings.enableDht) {
    throw PrayError.parse(
      "distribution config",
      "DHT announce is not implemented",
    );
  }
  if (settings.bootstrapTrackers.length > 0 && !allowsTorrent(settings)) {
    throw PrayError.parse(
      "distribution config",
      "bootstrap_trackers requires protocols to include torrent",
    );
  }
}
