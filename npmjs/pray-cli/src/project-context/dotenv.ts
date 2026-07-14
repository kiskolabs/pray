import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

const PRAY_ENVIRONMENT_PREFIX = "PRAY_";

export function loadDotenvVariables(projectRootHint: string): Map<string, string> {
  const path = join(projectRootHint, ".env");
  if (!existsSync(path)) {
    return new Map();
  }
  try {
    const text = readFileSync(path, "utf8");
    return parseDotenvText(text);
  } catch {
    return new Map();
  }
}

export function parseDotenvText(text: string): Map<string, string> {
  const variables = new Map<string, string>();
  for (const line of text.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (trimmed.length === 0 || trimmed.startsWith("#")) {
      continue;
    }
    const assignment = trimmed.startsWith("export ")
      ? trimmed.slice("export ".length)
      : trimmed;
    const separatorIndex = assignment.indexOf("=");
    if (separatorIndex < 0) {
      continue;
    }
    const key = assignment.slice(0, separatorIndex).trim();
    if (key.length === 0 || !key.startsWith(PRAY_ENVIRONMENT_PREFIX)) {
      continue;
    }
    const value = parseDotenvValue(assignment.slice(separatorIndex + 1).trim());
    variables.set(key, value);
  }
  return variables;
}

function parseDotenvValue(value: string): string {
  if (value.length >= 2) {
    const quote = value[0];
    if ((quote === '"' || quote === "'") && value.endsWith(quote)) {
      return value.slice(1, -1);
    }
  }
  return value;
}
