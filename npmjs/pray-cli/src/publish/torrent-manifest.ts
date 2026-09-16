import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import {
  allowsTorrent,
  defaultDistributionSettings,
  parseDistributionSettings,
  type RegistryDistributionSettings,
} from "../distribution.js";
import { PrayError } from "../errors.js";
import { sha256Prefixed } from "../hashing.js";
import { httpGetText, joinUrl } from "../http/client.js";

export const TORRENT_MANIFEST_SPEC = "pray-torrent-v1";
export const DEFAULT_TORRENT_PIECE_SIZE = 16 * 1024;

export interface TorrentManifestPayload {
  spec: string;
  name: string;
  version: string;
  artifact_url: string;
  artifact_hash: string;
  piece_size: number;
  length: number;
  pieces: string[];
  sources: string[];
  trackers: string[];
}

export function torrentManifestPath(artifactPath: string): string {
  return `${artifactPath}.praytorrent.json`;
}

export function torrentManifestPayload(
  name: string,
  version: string,
  artifactPath: string,
  archiveBytes: Buffer,
  trackers: string[],
): TorrentManifestPayload {
  return {
    spec: TORRENT_MANIFEST_SPEC,
    name,
    version,
    artifact_url: artifactPath,
    artifact_hash: sha256Prefixed(archiveBytes),
    piece_size: DEFAULT_TORRENT_PIECE_SIZE,
    length: archiveBytes.length,
    pieces: pieceHashes(archiveBytes),
    sources: [artifactPath],
    trackers,
  };
}

export function torrentManifestBytes(
  name: string,
  version: string,
  artifactPath: string,
  archiveBytes: Buffer,
  trackers: string[],
): string {
  return `${JSON.stringify(
    torrentManifestPayload(name, version, artifactPath, archiveBytes, trackers),
    null,
    2,
  )}\n`;
}

export function writeTorrentManifest(
  root: string,
  name: string,
  version: string,
  artifactPath: string,
  archiveBytes: Buffer,
  settings: RegistryDistributionSettings,
): void {
  if (!allowsTorrent(settings)) return;
  const path = join(root, torrentManifestPath(artifactPath));
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(
    path,
    torrentManifestBytes(
      name,
      version,
      artifactPath,
      archiveBytes,
      settings.bootstrapTrackers,
    ),
  );
}

export async function fetchDistributionSettings(
  sourceUrl: string,
): Promise<RegistryDistributionSettings> {
  try {
    return parseDistributionSettings(
      await httpGetText(joinUrl(sourceUrl, "v1/distribution.json")),
    );
  } catch (error) {
    if (isHttpNotFound(error)) {
      return defaultDistributionSettings();
    }
    throw error;
  }
}

export function torrentDescriptorPresent(
  root: string,
  artifactPath: string,
  settings: RegistryDistributionSettings,
): boolean {
  return (
    !allowsTorrent(settings) ||
    existsSync(join(root, torrentManifestPath(artifactPath)))
  );
}

function pieceHashes(archiveBytes: Buffer): string[] {
  if (archiveBytes.length === 0) return [];
  const hashes: string[] = [];
  for (
    let start = 0;
    start < archiveBytes.length;
    start += DEFAULT_TORRENT_PIECE_SIZE
  ) {
    hashes.push(
      sha256Prefixed(
        archiveBytes.subarray(start, start + DEFAULT_TORRENT_PIECE_SIZE),
      ),
    );
  }
  return hashes;
}

function isHttpNotFound(error: unknown): boolean {
  return error instanceof PrayError && /:\s*404\b/.test(error.message);
}
