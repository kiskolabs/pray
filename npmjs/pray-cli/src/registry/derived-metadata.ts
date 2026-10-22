import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { normalizeLineEndings } from "../hashing.js";
import type { PackageSpec } from "../package-spec/types.js";

const EMBEDDING_MODEL = "pray-hash-bucket-v1";
const EMBEDDING_DIMENSIONS = 16;
const MAX_SUMMARY_PARTS = 2;
const MAX_TOKEN_COUNT = 512;
const MAX_SNIPPET_LENGTH = 120;
const STOPWORDS = new Set([
  "a",
  "an",
  "and",
  "are",
  "as",
  "at",
  "be",
  "by",
  "for",
  "from",
  "has",
  "have",
  "in",
  "is",
  "it",
  "its",
  "of",
  "on",
  "or",
  "package",
  "the",
  "this",
  "to",
  "was",
  "we",
  "with",
  "you",
  "your",
]);

export type RegistryDerivedMetadata = {
  summary: string;
  topics: string[];
  categories: string[];
  possible_effects: string[];
  possible_side_effects: string[];
  embeddings: Array<{ model: string; vector: number[] }>;
  file_count: number;
  character_count: number;
  token_count: number;
};

export function derivedMetadataFromHash(
  value: unknown,
): RegistryDerivedMetadata | undefined {
  if (value == null || typeof value !== "object") return undefined;
  return value as RegistryDerivedMetadata;
}

export function deriveRegistryDerivedMetadataFromRoot(
  root: string,
  spec: PackageSpec,
): RegistryDerivedMetadata {
  const summaryCandidates: string[] = [];
  pushCandidate(summaryCandidates, spec.summary);
  pushCandidate(summaryCandidates, spec.description);
  for (const exportEntry of spec.exports.values()) {
    pushCandidate(summaryCandidates, exportEntry.summary);
  }

  let analyzedText = "";
  const sampleLines: string[] = [];
  let fileCount = 0;
  let characterCount = 0;
  for (const relative of spec.files) {
    const path = join(root, relative);
    if (!existsSync(path)) continue;
    let text: string;
    try {
      text = readFileSync(path, "utf8");
    } catch {
      continue;
    }
    const normalized = normalizeLineEndings(text);
    const line = firstMeaningfulLine(normalized);
    if (line) pushCandidate(sampleLines, line);
    fileCount += 1;
    characterCount += [...normalized].length;
    analyzedText += `${normalized}\n`;
  }

  for (const line of sampleLines) {
    pushCandidate(summaryCandidates, line);
  }
  if (summaryCandidates.length === 0) {
    for (const token of topTopics(analyzedText)) {
      pushCandidate(summaryCandidates, token);
    }
  }

  const summary = buildSummary(summaryCandidates);
  const tokens = tokenize(analyzedText);
  const topics = topTopics(analyzedText);
  const categories = inferCategories(
    `${summaryCandidates.join(" \n")} ${topics.join(" ")}`,
  );
  return {
    summary,
    topics,
    categories,
    possible_effects: inferPossibleEffects(categories),
    possible_side_effects: inferPossibleSideEffects(analyzedText, topics),
    embeddings: [{ model: EMBEDDING_MODEL, vector: hashedEmbedding(tokens) }],
    file_count: fileCount,
    character_count: characterCount,
    token_count: tokens.length,
  };
}

function buildSummary(candidates: string[]): string {
  const summary: string[] = [];
  const seen = new Set<string>();
  for (const candidate of candidates) {
    const normalized = candidate.trim();
    if (!normalized) continue;
    const key = normalized.toLowerCase();
    if (seen.has(key)) continue;
    seen.add(key);
    summary.push(truncate(normalized, MAX_SNIPPET_LENGTH));
    if (summary.length === MAX_SUMMARY_PARTS) break;
  }
  if (summary.length === 0) return "Package metadata";
  return summary.join(" — ");
}

function topTopics(text: string): string[] {
  const counts = new Map<string, number>();
  for (const token of tokenize(text)) {
    counts.set(token, (counts.get(token) ?? 0) + 1);
  }
  return [...counts.entries()]
    .sort(
      (left, right) => right[1] - left[1] || left[0].localeCompare(right[0]),
    )
    .slice(0, 5)
    .map(([token]) => token);
}

function inferCategories(text: string): string[] {
  const lowered = text.toLowerCase();
  const categories: string[] = [];
  if (
    containsAny(lowered, [
      "guide",
      "guidance",
      "readme",
      "documentation",
      "doc",
    ])
  ) {
    categories.push("documentation");
  }
  if (containsAny(lowered, ["test", "testing", "fixture", "spec"])) {
    categories.push("testing");
  }
  if (
    containsAny(lowered, ["auth", "security", "ssh", "key", "sign", "trust"])
  ) {
    categories.push("security");
  }
  if (
    containsAny(lowered, [
      "automation",
      "workflow",
      "render",
      "pipeline",
      "agent",
    ])
  ) {
    categories.push("automation");
  }
  if (categories.length === 0) categories.push("general");
  return [...new Set(categories)].sort();
}

function inferPossibleEffects(categories: string[]): string[] {
  const effects: string[] = [];
  if (categories.includes("documentation")) effects.push("clarifies usage");
  if (categories.includes("testing")) effects.push("improves validation");
  if (categories.includes("security")) {
    effects.push("surfaces trust-sensitive behavior");
  }
  if (categories.includes("automation")) effects.push("reduces manual steps");
  if (effects.length === 0) effects.push("improves package discovery");
  return effects;
}

function inferPossibleSideEffects(text: string, topics: string[]): string[] {
  const sideEffects: string[] = [];
  if (
    containsAny(text, [
      "write",
      "overwrite",
      "replace",
      "render",
      "inject",
      "delete",
    ])
  ) {
    sideEffects.push("may change generated output or managed files");
  }
  if (containsAny(topics.join(" "), ["publish", "release", "deploy"])) {
    sideEffects.push("may affect downstream package distribution");
  }
  return [...new Set(sideEffects)].sort();
}

function hashedEmbedding(tokens: string[]): number[] {
  const vector = Array.from({ length: EMBEDDING_DIMENSIONS }, () => 0);
  for (const token of tokens.slice(0, MAX_TOKEN_COUNT)) {
    const digest = createHash("sha256").update(token).digest();
    const bucket = digest[0]! % EMBEDDING_DIMENSIONS;
    vector[bucket]! += 1;
  }
  return vector;
}

function tokenize(text: string): string[] {
  const tokens: string[] = [];
  for (const raw of text.split(/[^\p{L}\p{N}]+/u)) {
    const token = raw.trim().toLowerCase();
    if (token.length < 4 || STOPWORDS.has(token)) continue;
    tokens.push(token);
    if (tokens.length === MAX_TOKEN_COUNT) break;
  }
  return tokens;
}

function containsAny(text: string, needles: string[]): boolean {
  return needles.some((needle) => text.includes(needle));
}

function pushCandidate(
  candidates: string[],
  candidate: string | undefined,
): void {
  const trimmed = candidate?.trim() ?? "";
  if (!trimmed) return;
  candidates.push(truncate(trimmed, MAX_SNIPPET_LENGTH));
}

function firstMeaningfulLine(text: string): string | undefined {
  for (const line of text.split("\n")) {
    const trimmed = line.trim();
    if (trimmed) return truncate(trimmed, MAX_SNIPPET_LENGTH);
  }
  return undefined;
}

function truncate(text: string, limit: number): string {
  return [...text].slice(0, limit).join("");
}
