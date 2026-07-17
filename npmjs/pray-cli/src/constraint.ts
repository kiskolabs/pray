import semver, { type Range, type SemVer } from "semver";
import { PrayError } from "./errors.js";

export function normalizeVersionConstraint(constraint: string): string {
  const trimmed = constraint.trim();
  if (trimmed.length === 0 || trimmed === "*") {
    return trimmed;
  }
  if (
    trimmed.startsWith("~>") ||
    trimmed.startsWith("~") ||
    trimmed.startsWith("^") ||
    trimmed.startsWith("=") ||
    trimmed.startsWith(">") ||
    trimmed.startsWith("<") ||
    trimmed.includes("*")
  ) {
    return trimmed;
  }
  if (semver.valid(trimmed)) {
    return `=${trimmed}`;
  }
  return trimmed;
}

export function versionSatisfies(version: string, constraint: string): boolean {
  const normalized = normalizeVersionConstraint(constraint);
  if (normalized.length === 0 || normalized === "*") {
    return true;
  }
  const parsedVersion = semver.parse(version);
  if (!parsedVersion) {
    throw PrayError.resolution(`invalid version ${version}`);
  }
  const requirement = normalized.trimStart().startsWith("~>")
    ? rubyPessimisticToSemver(normalized)
    : normalized.trim();
  const range = semver.validRange(requirement);
  if (!range) {
    throw PrayError.resolution(`invalid constraint ${constraint}`);
  }
  return semver.satisfies(parsedVersion, range);
}

export function pessimisticConstraintForVersion(version: string): string {
  const parsed = semver.parse(version);
  if (!parsed) {
    throw PrayError.resolution(`invalid version ${version}`);
  }
  if (parsed.minor === 0 && parsed.patch === 0) {
    return `~> ${parsed.major}.0`;
  }
  return `~> ${parsed.major}.${parsed.minor}`;
}

function rubyPessimisticToSemver(constraint: string): string {
  const text = constraint.trim().replace(/^~>\s*/, "");
  const parts = text.split(".");
  if (parts.length === 0 || parts.length > 3) {
    throw PrayError.resolution(
      `unsupported Ruby pessimistic constraint: ${constraint}`,
    );
  }
  const numbers = [0, 0, 0];
  for (let index = 0; index < parts.length; index += 1) {
    const parsed = Number.parseInt(parts[index]!, 10);
    if (Number.isNaN(parsed)) {
      throw PrayError.resolution(`invalid constraint segment ${parts[index]}`);
    }
    numbers[index] = parsed;
  }
  const first = numbers[0];
  const second = numbers[1];
  const third = numbers[2];
  if (first === undefined || second === undefined || third === undefined) {
    throw PrayError.resolution(
      `invalid constraint segment count in ${constraint}`,
    );
  }
  const lower = `${first}.${second}.${third}`;
  const upper =
    parts.length === 1 ? `${first + 1}.0.0` : `${first}.${second + 1}.0`;
  return `>=${lower} <${upper}`;
}

export type { Range, SemVer };
