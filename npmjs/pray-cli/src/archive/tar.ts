import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { PrayError } from "../errors.js";
import {
  MAX_ARCHIVE_ENTRIES,
  MAX_ARCHIVE_ENTRY_BYTES,
  MAX_ARCHIVE_TOTAL_BYTES,
} from "../resource-limits.js";
import { validateArchiveMemberPath } from "./path-safety.js";

const TAR_BLOCK_BYTES = 512;

export function validateTarArchive(bytes: Buffer): void {
  walkTarArchive(bytes);
}

export function extractTarArchive(
  bytes: Buffer,
  outputDirectory: string,
): void {
  mkdirSync(outputDirectory, { recursive: true });
  walkTarArchive(bytes, outputDirectory);
}

function walkTarArchive(bytes: Buffer, outputDirectory?: string): void {
  const paths = new Set<string>();
  let offset = 0;
  let entryCount = 0;
  let totalBytes = 0;
  let pendingPath: string | undefined;

  while (offset + TAR_BLOCK_BYTES <= bytes.length) {
    const header = bytes.subarray(offset, offset + TAR_BLOCK_BYTES);
    if (header.every((byte) => byte === 0)) return;
    verifyChecksum(header);
    const size = parseOctal(header.subarray(124, 136), "entry size");
    const dataStart = offset + TAR_BLOCK_BYTES;
    const dataEnd = dataStart + size;
    if (dataEnd > bytes.length) {
      throw PrayError.integrity("truncated package archive entry");
    }
    const type = String.fromCharCode(header[156] ?? 0);
    const data = bytes.subarray(dataStart, dataEnd);
    if (type === "x") {
      pendingPath = paxPath(data);
    } else if (type === "L") {
      pendingPath = cString(data).replace(/\n$/, "");
    } else {
      const rawPath = pendingPath ?? tarHeaderPath(header);
      pendingPath = undefined;
      if (type === "5" && (rawPath === "." || rawPath === "./")) {
        offset =
          dataStart + Math.ceil(size / TAR_BLOCK_BYTES) * TAR_BLOCK_BYTES;
        continue;
      }
      const path = validateArchiveMemberPath(rawPath);
      if (basenameStartsWithAppleDouble(path)) {
        offset =
          dataStart + Math.ceil(size / TAR_BLOCK_BYTES) * TAR_BLOCK_BYTES;
        continue;
      }
      if (type === "5") {
        if (outputDirectory !== undefined) {
          mkdirSync(extractedDestination(outputDirectory, path), {
            recursive: true,
          });
        }
      } else {
        if (type !== "0" && type !== "\0") {
          throw PrayError.integrity("unsupported package archive entry type");
        }
        entryCount += 1;
        if (entryCount > MAX_ARCHIVE_ENTRIES) {
          throw PrayError.integrity(
            `package archive exceeds ${MAX_ARCHIVE_ENTRIES} entries`,
          );
        }
        if (paths.has(path)) {
          throw PrayError.integrity(`duplicate package archive path: ${path}`);
        }
        paths.add(path);
        if (size > MAX_ARCHIVE_ENTRY_BYTES) {
          throw PrayError.integrity(
            `package archive entry exceeds ${MAX_ARCHIVE_ENTRY_BYTES} bytes: ${path}`,
          );
        }
        totalBytes += size;
        if (totalBytes > MAX_ARCHIVE_TOTAL_BYTES) {
          throw PrayError.integrity(
            `package archive exceeds ${MAX_ARCHIVE_TOTAL_BYTES} decompressed bytes`,
          );
        }
        if (outputDirectory !== undefined) {
          const destination = extractedDestination(outputDirectory, path);
          mkdirSync(dirname(destination), { recursive: true });
          writeFileSync(destination, data, { flag: "wx" });
        }
      }
    }
    offset = dataStart + Math.ceil(size / TAR_BLOCK_BYTES) * TAR_BLOCK_BYTES;
  }
  throw PrayError.integrity("package archive is missing its end marker");
}

function extractedDestination(outputDirectory: string, path: string): string {
  const destination = join(outputDirectory, path);
  if (
    destination !== outputDirectory &&
    !destination.startsWith(`${outputDirectory}/`) &&
    !destination.startsWith(`${outputDirectory}\\`)
  ) {
    throw PrayError.integrity(`package path escapes package root: ${path}`);
  }
  return destination;
}

function basenameStartsWithAppleDouble(path: string): boolean {
  const parts = path.split(/[/\\]/);
  const basename = parts[parts.length - 1] ?? "";
  return basename.startsWith("._");
}

function verifyChecksum(header: Buffer): void {
  // The checksum field is eight bytes. Writers fill it differently: six octal
  // digits then NUL and space, seven digits then NUL, or space padding. Read the
  // whole field and let parseOctal strip the terminator.
  const stored = parseOctal(header.subarray(148, 156), "checksum");
  let sum = 0;
  for (let index = 0; index < header.length; index += 1) {
    sum += index >= 148 && index < 156 ? 32 : (header[index] ?? 0);
  }
  if (sum !== stored) {
    throw PrayError.integrity("invalid package archive checksum");
  }
}

function tarHeaderPath(header: Buffer): string {
  const name = cString(header.subarray(0, 100));
  const prefix = cString(header.subarray(345, 500));
  return prefix.length > 0 ? `${prefix}/${name}` : name;
}

function cString(bytes: Buffer): string {
  const end = bytes.indexOf(0);
  return new TextDecoder("utf-8", { fatal: true }).decode(
    end >= 0 ? bytes.subarray(0, end) : bytes,
  );
}

function parseOctal(bytes: Buffer, label: string): number {
  const value = cString(bytes).trim();
  if (!/^[0-7]+$/.test(value)) {
    throw PrayError.integrity(`invalid package archive ${label}`);
  }
  const parsed = Number.parseInt(value, 8);
  if (!Number.isSafeInteger(parsed)) {
    throw PrayError.integrity(`invalid package archive ${label}`);
  }
  return parsed;
}

function paxPath(data: Buffer): string | undefined {
  let cursor = 0;
  let path: string | undefined;
  while (cursor < data.length) {
    const separator = data.indexOf(0x20, cursor);
    const length = Number.parseInt(
      data.subarray(cursor, separator).toString("ascii"),
      10,
    );
    const recordEnd = cursor + length;
    if (
      separator < 0 ||
      !Number.isSafeInteger(length) ||
      length <= 0 ||
      recordEnd > data.length
    ) {
      throw PrayError.integrity("invalid package archive pax header");
    }
    const record = data.subarray(separator + 1, recordEnd - 1);
    const equals = record.indexOf(0x3d);
    const key = equals < 0 ? "" : record.subarray(0, equals).toString("ascii");
    if (key === "path") {
      path = new TextDecoder("utf-8", { fatal: true }).decode(
        record.subarray(equals + 1),
      );
    }
    cursor = recordEnd;
  }
  return path;
}
