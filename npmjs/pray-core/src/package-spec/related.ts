import { PrayError } from "../errors.js";

export function canonicalRelated(references: string[]): string[] {
  const unique = [
    ...new Set(references.map((reference) => reference.trim()).sort()),
  ];
  for (const reference of unique) {
    if (!validReference(reference)) {
      throw PrayError.parse("prayspec", "invalid related prayer reference");
    }
  }
  return unique;
}

function validReference(reference: string): boolean {
  if (
    reference === "" ||
    reference.includes("\\") ||
    [...reference].some((character) => /\p{Cc}/u.test(character))
  ) {
    return false;
  }
  if (/^[A-Za-z][A-Za-z0-9+.-]*:.+/u.test(reference)) {
    return !uriHasUserInfo(reference);
  }
  if (reference.startsWith("/")) {
    const path = locatorPath(reference);
    return path.length > 1 && !path.startsWith("//");
  }
  if (reference.startsWith("./") || reference.startsWith("../")) {
    return locatorPath(reference)
      .split("/")
      .some((segment) => segment !== "" && segment !== "." && segment !== "..");
  }
  const segments = reference.split("/");
  return (
    !/[?#]/u.test(reference) &&
    segments.length === 2 &&
    segments.every(
      (segment) => segment !== "" && segment !== "." && segment !== "..",
    )
  );
}

function locatorPath(reference: string): string {
  return reference.split(/[?#]/u, 1)[0] ?? "";
}

function uriHasUserInfo(reference: string): boolean {
  const schemeSeparator = reference.indexOf("://");
  if (schemeSeparator === -1) return false;
  const authority = reference.slice(schemeSeparator + 3).split(/[/?#]/u, 1)[0];
  return authority?.includes("@") ?? false;
}
