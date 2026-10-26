import { PrayError } from "./errors.js";

export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function requireRecord(
  value: unknown,
  context: string,
): Record<string, unknown> {
  if (!isRecord(value)) {
    throw PrayError.parse(context, `expected object, got ${typeof value}`);
  }
  return value;
}

export function requireString(
  value: unknown,
  field: string,
  context: string,
): string {
  if (typeof value !== "string") {
    throw PrayError.parse(context, `${field} must be a string`);
  }
  return value;
}

export function requireBoolean(
  value: unknown,
  field: string,
  context: string,
): boolean {
  if (typeof value !== "boolean") {
    throw PrayError.parse(context, `${field} must be a boolean`);
  }
  return value;
}

export function requireNumber(
  value: unknown,
  field: string,
  context: string,
): number {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw PrayError.parse(context, `${field} must be a number`);
  }
  return value;
}

export function requireStringArray(
  value: unknown,
  field: string,
  context: string,
): string[] {
  if (!Array.isArray(value)) {
    throw PrayError.parse(context, `${field} must be an array`);
  }
  return value.map((entry, index) =>
    requireString(entry, `${field}[${index}]`, context),
  );
}

export function optionalString(
  value: unknown,
  field: string,
  context: string,
): string | undefined {
  if (value === undefined) {
    return undefined;
  }
  return requireString(value, field, context);
}
