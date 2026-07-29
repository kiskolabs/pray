import { isBalanced, splitTopLevel } from "./split.js";

export function expandStatementSurface(statement: string): string[] {
  const trimmed = statement.trim();
  if (trimmed.length === 0) {
    return [];
  }
  const parts: string[] = [];
  for (const segment of splitTopLevel(trimmed, ";")) {
    parts.push(...expandOneSurface(segment));
  }
  return parts;
}

function expandOneSurface(statement: string): string[] {
  const trimmed = statement.trim();
  if (trimmed.length === 0) {
    return [];
  }
  const braced = expandBraceBlock(trimmed);
  if (braced) {
    return braced;
  }
  return [normalizeKeywordCall(trimmed)];
}

function expandBraceBlock(statement: string): string[] | undefined {
  const keyword = leadingIdentifier(statement);
  if (!keyword) {
    return undefined;
  }
  const afterKeyword = statement.slice(keyword.length).trimStart();
  // Only keyword{…}, keyword(…){…}, or keyword "…"{…} — not spec.exports = {…}.
  const header = splitBraceHeader(afterKeyword);
  if (!header) {
    return undefined;
  }
  const { args, afterOpen } = header;
  const closeOffset = matchingCloseBrace(afterOpen);
  if (closeOffset === undefined) {
    return undefined;
  }
  const trailing = afterOpen.slice(closeOffset + 1).trim();
  if (trailing.length > 0) {
    return undefined;
  }
  const body = afterOpen.slice(0, closeOffset).trim();
  if (!isBalanced(body)) {
    return undefined;
  }

  const headerArgs = unwrapOuterParens(args);
  const open =
    headerArgs.length === 0 ? `${keyword} do` : `${keyword} ${headerArgs} do`;
  const output = [open];
  if (body.length > 0) {
    output.push(...expandStatementSurface(body));
  }
  output.push("end");
  return output;
}

function splitBraceHeader(
  afterKeyword: string,
): { args: string; afterOpen: string } | undefined {
  if (afterKeyword.startsWith("{")) {
    return { args: "", afterOpen: afterKeyword.slice(1) };
  }
  if (afterKeyword.startsWith("(")) {
    const close = matchingCloseParen(afterKeyword);
    if (close === undefined) {
      return undefined;
    }
    const trailing = afterKeyword.slice(close + 1).trimStart();
    if (!trailing.startsWith("{")) {
      return undefined;
    }
    return {
      args: afterKeyword.slice(1, close).trim(),
      afterOpen: trailing.slice(1),
    };
  }
  const first = afterKeyword[0];
  if (first !== '"' && first !== "'" && first !== ":") {
    return undefined;
  }
  const braceOffset = findTopLevelChar(afterKeyword, "{");
  if (braceOffset === undefined) {
    return undefined;
  }
  return {
    args: afterKeyword.slice(0, braceOffset).trim(),
    afterOpen: afterKeyword.slice(braceOffset + 1),
  };
}

function normalizeKeywordCall(statement: string): string {
  const spaced = normalizeSpacedBlockOpener(statement);
  if (spaced) {
    return spaced;
  }
  const keyword = leadingIdentifier(statement);
  if (!keyword) {
    return statement;
  }
  const afterKeyword = statement.slice(keyword.length).trimStart();
  if (!afterKeyword.startsWith("(")) {
    return statement;
  }
  const close = matchingCloseParen(afterKeyword);
  if (close === undefined) {
    return statement;
  }
  const inner = afterKeyword.slice(1, close).trim();
  const trailing = afterKeyword.slice(close + 1).trim();
  return trailing.length === 0
    ? `${keyword} ${inner}`
    : `${keyword} ${inner} ${trailing}`;
}

function normalizeSpacedBlockOpener(statement: string): string | undefined {
  const trimmed = statement.trim();
  for (const keyword of ["pray", "template"]) {
    if (!trimmed.startsWith(keyword)) {
      continue;
    }
    const rest = trimmed.slice(keyword.length).trimStart();
    if (rest === "do") {
      return `${keyword} do`;
    }
  }
  return undefined;
}

export function splitSymbolAssignment(
  statement: string,
): { key: string; value: string } | undefined {
  const trimmed = statement.trim();
  const call = splitSymbolCall(trimmed);
  if (call) {
    return call;
  }
  const match = trimmed.match(/^(\S+)\s+(.+)$/);
  if (!match) {
    return undefined;
  }
  const key = match[1]?.trim() ?? "";
  const value = match[2]?.trim() ?? "";
  if (key.length === 0 || value.length === 0) {
    return undefined;
  }
  return { key, value };
}

function splitSymbolCall(
  statement: string,
): { key: string; value: string } | undefined {
  const key = leadingIdentifier(statement);
  if (!key) {
    return undefined;
  }
  const afterKey = statement.slice(key.length).trimStart();
  if (!afterKey.startsWith("(") || !afterKey.endsWith(")")) {
    return undefined;
  }
  if (matchingCloseParen(afterKey) !== afterKey.length - 1) {
    return undefined;
  }
  const inner = afterKey.slice(1, -1).trim();
  if (inner.length === 0) {
    return undefined;
  }
  return { key, value: inner };
}

function leadingIdentifier(input: string): string | undefined {
  const trimmed = input.trimStart();
  let end = 0;
  while (end < trimmed.length) {
    const character = trimmed[end];
    if (
      character === undefined ||
      !(
        (character >= "a" && character <= "z") ||
        (character >= "A" && character <= "Z") ||
        (character >= "0" && character <= "9") ||
        character === "_"
      )
    ) {
      break;
    }
    end += 1;
  }
  if (end === 0) {
    return undefined;
  }
  const ident = trimmed.slice(0, end);
  const first = ident[0];
  if (
    first === undefined ||
    !((first >= "a" && first <= "z") || (first >= "A" && first <= "Z"))
  ) {
    return undefined;
  }
  return ident;
}

function unwrapOuterParens(input: string): string {
  const trimmed = input.trim();
  if (
    trimmed.startsWith("(") &&
    matchingCloseParen(trimmed) === trimmed.length - 1
  ) {
    return trimmed.slice(1, -1).trim();
  }
  return trimmed;
}

function matchingCloseParen(input: string): number | undefined {
  return matchingCloseDelimited(input, "(", ")");
}

function matchingCloseBrace(input: string): number | undefined {
  let depth = 1;
  let quote: string | undefined;
  let escaped = false;
  for (let index = 0; index < input.length; index += 1) {
    const character = input[index];
    if (character === undefined) {
      break;
    }
    if (quote) {
      if (escaped) {
        escaped = false;
      } else if (character === "\\") {
        escaped = true;
      } else if (character === quote) {
        quote = undefined;
      }
      continue;
    }
    if (character === '"' || character === "'") {
      quote = character;
      continue;
    }
    if (character === "{") {
      depth += 1;
      continue;
    }
    if (character === "}") {
      depth -= 1;
      if (depth === 0) {
        return index;
      }
    }
  }
  return undefined;
}

function matchingCloseDelimited(
  input: string,
  open: string,
  close: string,
): number | undefined {
  if (!input.startsWith(open)) {
    return undefined;
  }
  let depth = 0;
  let quote: string | undefined;
  let escaped = false;
  for (let index = 0; index < input.length; index += 1) {
    const character = input[index];
    if (character === undefined) {
      break;
    }
    if (quote) {
      if (escaped) {
        escaped = false;
      } else if (character === "\\") {
        escaped = true;
      } else if (character === quote) {
        quote = undefined;
      }
      continue;
    }
    if (character === open) {
      depth += 1;
    } else if (character === close) {
      depth -= 1;
      if (depth === 0) {
        return index;
      }
    }
  }
  return undefined;
}

function findTopLevelChar(input: string, needle: string): number | undefined {
  let depth = 0;
  let quote: string | undefined;
  let escaped = false;
  for (let index = 0; index < input.length; index += 1) {
    const character = input[index];
    if (character === undefined) {
      break;
    }
    if (quote) {
      if (escaped) {
        escaped = false;
      } else if (character === "\\") {
        escaped = true;
      } else if (character === quote) {
        quote = undefined;
      }
      continue;
    }
    if (character === '"' || character === "'") {
      quote = character;
      continue;
    }
    if (character === "(" || character === "[" || character === "{") {
      if (depth === 0 && character === needle) {
        return index;
      }
      depth += 1;
      continue;
    }
    if (character === ")" || character === "]" || character === "}") {
      depth -= 1;
      continue;
    }
    if (depth === 0 && character === needle) {
      return index;
    }
  }
  return undefined;
}

export class SurfaceStatementReader {
  private readonly pending: string[] = [];

  pushRaw(statement: string): void {
    this.pending.push(...expandStatementSurface(statement));
  }

  next(): string | undefined {
    return this.pending.shift();
  }
}
