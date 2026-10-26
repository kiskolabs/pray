import { parse } from "smol-toml";
import { PrayError } from "../errors.js";
import { sha256Prefixed } from "../hashing.js";
import { parseLockfileValue } from "./parse.js";
import { serializeLockfileText } from "./serialize.js";
import type { Lockfile } from "./types.js";

/** Parse lockfile TOML text into a {@link Lockfile}. */
export function parseLockfile(text: string): Lockfile {
  try {
    return parseLockfileValue(parse(text));
  } catch (error) {
    if (error instanceof PrayError) {
      throw error;
    }
    const message = error instanceof Error ? error.message : String(error);
    throw PrayError.parse("lockfile", message);
  }
}

/** True when two lockfiles serialize to the same canonical TOML. */
export function lockfilesEquivalent(left: Lockfile, right: Lockfile): boolean {
  return serializeLockfile(left) === serializeLockfile(right);
}

/** Serialize a lockfile to canonical TOML text. */
export function serializeLockfile(lockfile: Lockfile): string {
  return serializeLockfileText(lockfile);
}

/** SHA-256 hash of the canonical lockfile serialization. */
export function lockfileHash(lockfile: Lockfile): string {
  return sha256Prefixed(serializeLockfile(lockfile));
}
