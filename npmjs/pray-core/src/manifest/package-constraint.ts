import { PrayError } from "../errors.js";

const PACKAGE_KEYWORDS = ["pray", "use", "include", "agent", "package"];

export function rewriteConstraintOnLine(
  line: string,
  constraint: string,
): string {
  const indentLength = line.length - line.trimStart().length;
  const indent = line.slice(0, indentLength);
  const trimmed = line.trimStart();
  const afterKeyword = skipPackageKeyword(trimmed);
  if (afterKeyword === undefined) {
    throw PrayError.manifest("package declaration is missing a keyword");
  }
  const afterKeywordTrimmed = afterKeyword.trimStart();
  const parsedName = parseQuoted(afterKeywordTrimmed);
  if (!parsedName) {
    throw PrayError.manifest("package declaration is missing a quoted name");
  }
  const [name, afterName] = parsedName;
  const quotedConstraint = `"${constraint}"`;
  const keywordAndName = trimmed.slice(0, trimmed.length - afterName.length);
  const remainder = afterName.trimStart();
  if (remainder.length === 0) {
    return `${indent}${keywordAndName}, ${quotedConstraint}`;
  }
  if (!remainder.startsWith(",")) {
    throw PrayError.manifest(
      `package ${name} declaration is missing a comma after the name`,
    );
  }
  const afterComma = remainder.slice(1).trimStart();
  if (afterComma.startsWith('"') || afterComma.startsWith("'")) {
    const parsedConstraint = parseQuoted(afterComma);
    if (!parsedConstraint) {
      throw PrayError.manifest(
        `package ${name} declaration has an unclosed constraint`,
      );
    }
    return `${indent}${keywordAndName}, ${quotedConstraint}${parsedConstraint[1]}`;
  }
  return `${indent}${keywordAndName}, ${quotedConstraint}, ${afterComma}`;
}

function skipPackageKeyword(input: string): string | undefined {
  for (const keyword of PACKAGE_KEYWORDS) {
    if (!input.startsWith(keyword)) {
      continue;
    }
    const rest = input.slice(keyword.length);
    const next = rest[0];
    if (next === undefined || next === '"' || next === "'" || /\s/.test(next)) {
      return rest;
    }
  }
  return undefined;
}

function parseQuoted(input: string): [string, string] | undefined {
  const quote = input[0];
  if (quote !== '"' && quote !== "'") {
    return undefined;
  }
  const rest = input.slice(1);
  const end = rest.indexOf(quote);
  if (end === -1) {
    return undefined;
  }
  return [rest.slice(0, end), rest.slice(end + 1)];
}
