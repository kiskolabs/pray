import { PrayError } from "../errors.js";
import type { LiteralValue } from "./types.js";

export function parseLiteral(input: string): LiteralValue {
  const parser = new LiteralParser(input);
  const value = parser.parseValue();
  parser.skipWhitespace();
  if (!parser.isFinished()) {
    throw PrayError.parse(
      "literal",
      `unexpected trailing input near ${JSON.stringify(parser.remaining())}`,
    );
  }
  return value;
}

export function parseLiteralMap(input: string): Map<string, LiteralValue> {
  const value = parseLiteral(input);
  if (value.kind !== "map") {
    throw PrayError.parse(
      "literal",
      `expected map literal, found ${value.kind}`,
    );
  }
  return value.value;
}

export function parseLiteralArray(input: string): LiteralValue[] {
  const value = parseLiteral(input);
  if (value.kind !== "array") {
    throw PrayError.parse(
      "literal",
      `expected array literal, found ${value.kind}`,
    );
  }
  return value.value;
}

class LiteralParser {
  private cursor = 0;

  constructor(private readonly input: string) {}

  isFinished(): boolean {
    return this.cursor >= this.input.length;
  }

  remaining(): string {
    return this.input.slice(this.cursor);
  }

  private peek(): string | undefined {
    return this.input[this.cursor];
  }

  private next(): string | undefined {
    const character = this.peek();
    if (character !== undefined) {
      this.cursor += 1;
    }
    return character;
  }

  skipWhitespace(): void {
    while (this.peek() !== undefined && /\s/.test(this.peek()!)) {
      this.cursor += 1;
    }
  }

  parseValue(): LiteralValue {
    this.skipWhitespace();
    const character = this.peek();
    if (character === '"' || character === "'") {
      return this.parseString();
    }
    if (character === ":") {
      return this.parseSymbol();
    }
    if (character === "[") {
      return this.parseArray();
    }
    if (character === "{") {
      return this.parseMap();
    }
    if (
      character !== undefined &&
      (/\d/.test(character) || character === "-")
    ) {
      return this.parseIntegerOrIdentifier();
    }
    if (character !== undefined) {
      return this.parseIdentifier();
    }
    throw PrayError.parse("literal", "unexpected end of input");
  }

  private parseString(): LiteralValue {
    const quote = this.next()!;
    let output = "";
    let escaped = false;
    while (this.peek() !== undefined) {
      const character = this.next()!;
      if (escaped) {
        output += unescapeCharacter(character);
        escaped = false;
        continue;
      }
      if (character === "\\") {
        escaped = true;
        continue;
      }
      if (character === quote) {
        return { kind: "string", value: output };
      }
      output += character;
    }
    throw PrayError.parse("literal", "unterminated string literal");
  }

  private parseSymbol(): LiteralValue {
    this.next();
    let output = "";
    while (this.peek() !== undefined) {
      const character = this.peek()!;
      if (/[\w\-./]/.test(character)) {
        output += this.next();
      } else {
        break;
      }
    }
    if (output.length === 0) {
      throw PrayError.parse("literal", "empty symbol");
    }
    return { kind: "symbol", value: output };
  }

  private parseArray(): LiteralValue {
    this.next();
    const values: LiteralValue[] = [];
    while (true) {
      this.skipWhitespace();
      if (this.peek() === "]") {
        this.next();
        break;
      }
      values.push(this.parseValue());
      this.skipWhitespace();
      if (this.peek() === ",") {
        this.next();
        continue;
      }
      if (this.peek() === "]") {
        this.next();
        break;
      }
      throw PrayError.parse("literal", "expected ',' or ']'");
    }
    return { kind: "array", value: values };
  }

  private parseMap(): LiteralValue {
    this.next();
    const entries = new Map<string, LiteralValue>();
    while (true) {
      this.skipWhitespace();
      if (this.peek() === "}") {
        this.next();
        break;
      }
      const key = this.parseMapKey();
      this.skipWhitespace();
      if (this.remaining().startsWith("=>")) {
        this.cursor += 2;
      } else if (this.peek() === ":") {
        this.next();
      } else {
        throw PrayError.parse("literal", "expected ':' or '=>' after map key");
      }
      const value = this.parseValue();
      entries.set(key, value);
      this.skipWhitespace();
      if (this.peek() === ",") {
        this.next();
        continue;
      }
      if (this.peek() === "}") {
        this.next();
        break;
      }
      throw PrayError.parse("literal", "expected ',' or '}'");
    }
    return { kind: "map", value: entries };
  }

  private parseMapKey(): string {
    this.skipWhitespace();
    const character = this.peek();
    if (character === '"' || character === "'") {
      const value = this.parseString();
      if (value.kind !== "string") {
        throw PrayError.parse("literal", "invalid map key");
      }
      return value.value;
    }
    if (character === ":") {
      const value = this.parseSymbol();
      if (value.kind !== "symbol") {
        throw PrayError.parse("literal", "invalid map key");
      }
      return value.value;
    }
    if (character !== undefined && isIdentifierStart(character)) {
      return this.parseIdentifierName();
    }
    throw PrayError.parse("literal", "invalid map key");
  }

  private parseIntegerOrIdentifier(): LiteralValue {
    const start = this.cursor;
    if (this.peek() === "-") {
      this.next();
    }
    while (this.peek() !== undefined && /[\d_]/.test(this.peek()!)) {
      this.next();
    }
    if (this.peek() === ".") {
      return this.parseIdentifierFrom(start);
    }
    const text = this.input.slice(start, this.cursor).replace(/_/g, "");
    const parsed = Number.parseInt(text, 10);
    if (Number.isNaN(parsed)) {
      throw PrayError.parse("literal", `invalid integer ${text}`);
    }
    return { kind: "integer", value: parsed };
  }

  private parseIdentifier(): LiteralValue {
    const identifier = this.parseIdentifierName();
    if (identifier === "true") {
      return { kind: "bool", value: true };
    }
    if (identifier === "false") {
      return { kind: "bool", value: false };
    }
    if (identifier === "nil") {
      return { kind: "null" };
    }
    return { kind: "string", value: identifier };
  }

  private parseIdentifierName(): string {
    const start = this.cursor;
    if (this.peek() === undefined || !isIdentifierStart(this.peek()!)) {
      throw PrayError.parse("literal", "expected identifier");
    }
    this.next();
    while (this.peek() !== undefined && isIdentifierContinue(this.peek()!)) {
      this.next();
    }
    return this.input.slice(start, this.cursor);
  }

  private parseIdentifierFrom(start: number): LiteralValue {
    while (
      this.peek() !== undefined &&
      (isIdentifierContinue(this.peek()!) || this.peek() === ".")
    ) {
      this.next();
    }
    return { kind: "string", value: this.input.slice(start, this.cursor) };
  }
}

function unescapeCharacter(character: string): string {
  switch (character) {
    case "n":
      return "\n";
    case "r":
      return "\r";
    case "t":
      return "\t";
    case "\\":
      return "\\";
    case '"':
      return '"';
    case "'":
      return "'";
    default:
      return character;
  }
}

function isIdentifierStart(character: string): boolean {
  return /[A-Za-z_]/.test(character);
}

function isIdentifierContinue(character: string): boolean {
  return /[\w\-./]/.test(character);
}
