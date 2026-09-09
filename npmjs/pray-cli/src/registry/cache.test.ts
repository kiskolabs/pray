import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { describe, it } from "node:test";
import { fileURLToPath } from "node:url";
import { PrayError } from "../errors.js";
import { registryCacheDirectory } from "./cache.js";

type CacheFixture = {
  source_key: string;
  package_name: string;
  version: string;
  relative_path: string;
};

const here = dirname(fileURLToPath(import.meta.url));

describe("registry cache identity", () => {
  it("matches the shared cache identity fixture", () => {
    const fixture = JSON.parse(
      readFileSync(
        join(
          here,
          "../../../../testdata/shared/registry-cache/identity-first.json",
        ),
        "utf8",
      ),
    ) as CacheFixture;

    assert.equal(
      registryCacheDirectory(
        "/project",
        fixture.source_key,
        fixture.package_name,
        fixture.version,
      ),
      join("/project", fixture.relative_path),
    );
  });

  it("rejects unsafe package and version segments", () => {
    for (const packageName of [
      "sample",
      "sample/base/extra",
      "sample//base",
      "./base",
      "../base",
      "sample/..",
      String.raw`sample\base`,
    ]) {
      assert.throws(
        () =>
          registryCacheDirectory("/project", "source", packageName, "1.0.0"),
        (error: unknown) => error instanceof PrayError,
      );
    }

    for (const version of ["", ".", "..", "1/2", String.raw`1\2`]) {
      assert.throws(
        () =>
          registryCacheDirectory("/project", "source", "sample/base", version),
        (error: unknown) => error instanceof PrayError,
      );
    }
  });
});
