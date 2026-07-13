import { PrayError } from "../errors.js";
import { parseLiteral } from "./parser.js";
import { findTopLevel, splitTopLevel } from "./split.js";
import {
  literalAsArray,
  literalAsString,
  type LiteralValue,
} from "./types.js";

export interface ParsedCall {
  values: LiteralValue[];
  keywords: Map<string, LiteralValue>;
}

export function parseCall(rest: string): ParsedCall {
  const positional: LiteralValue[] = [];
  const keywords = new Map<string, LiteralValue>();
  for (const segment of splitTopLevel(rest.trim().replace(/,\s*$/, ""), ",")) {
    const keyword = parseKeywordSegment(segment);
    if (keyword) {
      keywords.set(keyword.key, keyword.value);
    } else if (segment.length > 0) {
      positional.push(parseLiteral(segment));
    }
  }
  return { values: positional, keywords };
}

function parseKeywordSegment(
  segment: string,
): { key: string; value: LiteralValue } | undefined {
  const arrowIndex = findTopLevel(segment, "=>");
  if (arrowIndex !== undefined) {
    const key = stringFromLiteral(segment.slice(0, arrowIndex).trim(), "literal");
    const value = parseLiteral(segment.slice(arrowIndex + 2).trim());
    return { key, value };
  }
  const colonIndex = findTopLevel(segment, ":");
  if (colonIndex !== undefined) {
    const left = segment.slice(0, colonIndex).trim();
    const right = segment.slice(colonIndex + 1).trim();
    if (left.length === 0) {
      return undefined;
    }
    const key = left.replace(/^:/, "");
    return { key, value: parseLiteral(right) };
  }
  return undefined;
}

export function keywordArray(
  keywords: Map<string, LiteralValue>,
  key: string,
): string[] {
  const value = keywords.get(key);
  if (!value) {
    return [];
  }
  const array = literalAsArray(value);
  if (!array) {
    return [];
  }
  return array
    .map((entry) => literalAsString(entry))
    .filter((entry): entry is string => entry !== undefined);
}

export function stringFromValue(value: LiteralValue, context: string): string {
  const text = literalAsString(value);
  if (text === undefined) {
    throw PrayError.parse(context, "expected string-like literal");
  }
  return text;
}

export function stringFromLiteral(input: string, context: string): string {
  return stringFromValue(parseLiteral(input), context);
}

export function requirePositionalString(
  values: LiteralValue[],
  index: number,
  context: string,
): string {
  const value = values[index];
  if (value === undefined) {
    throw PrayError.parse(context, `missing positional argument at index ${index}`);
  }
  return stringFromValue(value, context);
}

export function keywordValue(
  keywords: Map<string, LiteralValue>,
  key: string,
  context: string,
): LiteralValue {
  const value = keywords.get(key);
  if (value === undefined) {
    throw PrayError.parse(context, `missing keyword ${key}`);
  }
  return value;
}
