export type LiteralValue =
  | { kind: "string"; value: string }
  | { kind: "symbol"; value: string }
  | { kind: "bool"; value: boolean }
  | { kind: "null" }
  | { kind: "integer"; value: number }
  | { kind: "array"; value: LiteralValue[] }
  | { kind: "map"; value: Map<string, LiteralValue> };

export function literalAsString(value: LiteralValue): string | undefined {
  if (value.kind === "string" || value.kind === "symbol") {
    return value.value;
  }
  return undefined;
}

export function literalAsBool(value: LiteralValue): boolean | undefined {
  return value.kind === "bool" ? value.value : undefined;
}

export function literalAsInteger(value: LiteralValue): number | undefined {
  return value.kind === "integer" ? value.value : undefined;
}

export function literalAsArray(
  value: LiteralValue,
): LiteralValue[] | undefined {
  return value.kind === "array" ? value.value : undefined;
}

export function literalAsMap(
  value: LiteralValue,
): Map<string, LiteralValue> | undefined {
  return value.kind === "map" ? value.value : undefined;
}
