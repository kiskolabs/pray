import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { PrayError } from "../errors.js";
import { unpackPraypkg } from "./praypkg.js";

describe("praypkg extraction", () => {
  it("rejects duplicate archive paths", () => {
    const output = mkdtempSync(join(tmpdir(), "pray-archive-duplicate-"));
    const tarBytes = Buffer.concat([
      tarEntry("rules.md", Buffer.from("first\n")),
      tarEntry("rules.md", Buffer.from("second\n")),
      Buffer.alloc(1024),
    ]);
    const compressed = spawnSync("zstd", ["-q", "-c"], { input: tarBytes });
    try {
      assert.throws(
        () => unpackPraypkg(compressed.stdout, output),
        (error: unknown) =>
          error instanceof PrayError && error.message.includes("duplicate"),
      );
    } finally {
      rmSync(output, { recursive: true, force: true });
    }
  });

  it("rejects parent directory escape paths and invalid checksums", () => {
    const output = mkdtempSync(join(tmpdir(), "pray-archive-escape-"));
    try {
      const escaped = spawnSync("zstd", ["-q", "-c"], {
        input: Buffer.concat([
          tarEntry("../escape.md", Buffer.from("owned\n")),
          Buffer.alloc(1024),
        ]),
      });
      assert.throws(
        () => unpackPraypkg(escaped.stdout, output),
        (error: unknown) =>
          error instanceof PrayError && error.message.includes("escapes"),
      );

      const header = tarEntry("rules.md", Buffer.from("ok\n"));
      header[0] = header[0]! ^ 0xff;
      const checksum = spawnSync("zstd", ["-q", "-c"], {
        input: Buffer.concat([header, Buffer.alloc(1024)]),
      });
      assert.throws(
        () => unpackPraypkg(checksum.stdout, output),
        (error: unknown) =>
          error instanceof PrayError && error.message.includes("checksum"),
      );
    } finally {
      rmSync(output, { recursive: true, force: true });
    }
  });

  it("skips AppleDouble sidecar members", () => {
    const output = mkdtempSync(join(tmpdir(), "pray-archive-apple-"));
    const compressed = spawnSync("zstd", ["-q", "-c"], {
      input: Buffer.concat([
        tarEntry("demo.prayspec", Buffer.from("ok\n")),
        tarEntry("._demo.prayspec", Buffer.from("sidecar\n")),
        Buffer.alloc(1024),
      ]),
    });
    try {
      unpackPraypkg(compressed.stdout, output);
      assert.equal(existsSync(join(output, "demo.prayspec")), true);
      assert.equal(existsSync(join(output, "._demo.prayspec")), false);
    } finally {
      rmSync(output, { recursive: true, force: true });
    }
  });

  it("accepts checksum fields written in other tar conventions", () => {
    for (const writeChecksum of [sevenDigitChecksum, paddedChecksum]) {
      const output = mkdtempSync(join(tmpdir(), "pray-archive-checksum-"));
      const compressed = spawnSync("zstd", ["-q", "-c"], {
        input: Buffer.concat([
          tarEntry("rules.md", Buffer.from("ok\n"), writeChecksum),
          Buffer.alloc(1024),
        ]),
      });
      try {
        unpackPraypkg(compressed.stdout, output);
        assert.equal(existsSync(join(output, "rules.md")), true);
      } finally {
        rmSync(output, { recursive: true, force: true });
      }
    }
  });
});

// The eight byte checksum field is filled differently by different tar writers.
type ChecksumWriter = (checksum: number) => string;

const posixChecksum: ChecksumWriter = (checksum) =>
  `${checksum.toString(8).padStart(6, "0")}\0 `;
const sevenDigitChecksum: ChecksumWriter = (checksum) =>
  `${checksum.toString(8).padStart(7, "0")}\0`;
const paddedChecksum: ChecksumWriter = (checksum) =>
  `${checksum.toString(8).padStart(7, " ")}\0`;

function tarEntry(
  path: string,
  content: Buffer,
  writeChecksum: ChecksumWriter = posixChecksum,
): Buffer {
  const header = Buffer.alloc(512);
  header.write(path, 0, 100, "utf8");
  header.write("0000644\0", 100, "ascii");
  header.write("0000000\0", 108, "ascii");
  header.write("0000000\0", 116, "ascii");
  header.write(
    `${content.length.toString(8).padStart(11, "0")}\0`,
    124,
    "ascii",
  );
  header.write("00000000000\0", 136, "ascii");
  header.fill(" ", 148, 156);
  header.write("0", 156, "ascii");
  header.write("ustar\0", 257, "ascii");
  header.write("00", 263, "ascii");
  const checksum = [...header].reduce((sum, byte) => sum + byte, 0);
  header.write(writeChecksum(checksum), 148, "ascii");
  const padding = Buffer.alloc((512 - (content.length % 512)) % 512);
  return Buffer.concat([header, content, padding]);
}
