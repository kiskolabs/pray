import type { PackageExportKind } from "../domain/types.js";
import { PrayError } from "../errors.js";
import {
  keywordValue,
  parseCall,
  requirePositionalString,
  stringFromValue,
} from "../literal/call-parser.js";
import { parseLiteral, parseLiteralMap } from "../literal/parser.js";
import {
  literalAsArray,
  literalAsBool,
  literalAsString,
  type LiteralValue,
} from "../literal/types.js";
import type {
  PackageDependency,
  PackageExport,
  PackageSkill,
  PackageTemplate,
} from "./types.js";

const PARSE_CONTEXT = "prayspec";

export function parseDependency(rest: string, optional: boolean): PackageDependency {
  const { values, keywords } = parseCall(rest);
  return {
    name: requirePositionalString(values, 0, PARSE_CONTEXT),
    constraint: values[1]
      ? stringFromValue(values[1], PARSE_CONTEXT)
      : "*",
    optional: keywords.has("optional")
      ? literalAsBool(keywordValue(keywords, "optional", PARSE_CONTEXT)) ?? optional
      : optional,
  };
}

export function parseExports(value: string): Map<string, PackageExport> {
  const map = parseLiteralMap(value);
  const exports = new Map<string, PackageExport>();
  for (const [name, literal] of map.entries()) {
    const entry = literalAsMap(literal);
    if (!entry) {
      throw PrayError.parse(PARSE_CONTEXT, `export ${name} must be a map`);
    }
    const path = mapString(entry, "path");
    if (!path) {
      throw PrayError.parse(PARSE_CONTEXT, `export ${name} missing path`);
    }
    exports.set(name, {
      kind: (mapString(entry, "type") ?? "fragment") as PackageExportKind,
      path,
      ...(mapString(entry, "summary")
        ? { summary: mapString(entry, "summary") }
        : {}),
    });
  }
  return exports;
}

export function parseSkills(value: string): Map<string, PackageSkill> {
  const map = parseLiteralMap(value);
  const skills = new Map<string, PackageSkill>();
  for (const [name, literal] of map.entries()) {
    const entry = literalAsMap(literal);
    if (!entry) {
      throw PrayError.parse(PARSE_CONTEXT, `skill ${name} must be a map`);
    }
    const path = mapString(entry, "path");
    if (!path) {
      throw PrayError.parse(PARSE_CONTEXT, "skill missing path");
    }
    skills.set(name, {
      path,
      ...(mapString(entry, "summary")
        ? { summary: mapString(entry, "summary") }
        : {}),
    });
  }
  return skills;
}

export function parseTemplates(value: string): Map<string, PackageTemplate> {
  const map = parseLiteralMap(value);
  const templates = new Map<string, PackageTemplate>();
  for (const [name, literal] of map.entries()) {
    const entry = literalAsMap(literal);
    if (!entry) {
      throw PrayError.parse(PARSE_CONTEXT, `template ${name} must be a map`);
    }
    const path = mapString(entry, "path");
    if (!path) {
      throw PrayError.parse(PARSE_CONTEXT, "template missing path");
    }
    templates.set(name, {
      path,
      ...(mapString(entry, "summary")
        ? { summary: mapString(entry, "summary") }
        : {}),
    });
  }
  return templates;
}

export function parseStringMap(value: string): Map<string, string> {
  const map = parseLiteralMap(value);
  const output = new Map<string, string>();
  for (const [key, literal] of map.entries()) {
    const text = literalAsString(literal);
    if (!text) {
      throw PrayError.parse(PARSE_CONTEXT, `expected string value for ${key}`);
    }
    output.set(key, text);
  }
  return output;
}

export function arrayOfStrings(value: string): string[] {
  const array = parseLiteral(value);
  const entries = literalAsArray(array);
  if (!entries) {
    throw PrayError.parse(PARSE_CONTEXT, "expected array literal");
  }
  return entries.map((entry) => stringFromValue(entry, PARSE_CONTEXT));
}

function literalAsMap(
  value: LiteralValue,
): Map<string, LiteralValue> | undefined {
  return value.kind === "map" ? value.value : undefined;
}

function mapString(map: Map<string, LiteralValue>, key: string): string | undefined {
  const value = map.get(key);
  return value ? literalAsString(value) : undefined;
}
