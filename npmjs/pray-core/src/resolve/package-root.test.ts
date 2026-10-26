import assert from "node:assert/strict";
import { describe, it } from "node:test";
import type { ManifestPackage, ManifestSource } from "../manifest/types.js";
import { impliedSourceName } from "./package-root.js";

function packageEntry(name: string, source?: string): ManifestPackage {
  return {
    name,
    constraint: "*",
    exports: [],
    targets: [],
    features: [],
    groups: [],
    optional: false,
    ...(source ? { source } : {}),
  };
}

function source(
  name: string,
  kind: ManifestSource["kind"],
  url: string,
): ManifestSource {
  return { name, kind, url };
}

describe("impliedSourceName", () => {
  it("uses the unique path source for an unqualified name", () => {
    const sources = new Map([
      [
        "amkisko",
        source("amkisko", "git", "git+https://example.com/prayers.git"),
      ],
      ["local", source("local", "path", "prayers")],
    ]);
    assert.equal(impliedSourceName(packageEntry("project"), sources), "local");
  });

  it("uses a namespaced name to pick a path source when several exist", () => {
    const sources = new Map([
      ["local", source("local", "path", "prayers")],
      ["vendor", source("vendor", "path", "vendor")],
    ]);
    assert.equal(
      impliedSourceName(packageEntry("local/project"), sources),
      "local",
    );
  });

  it("still matches a git namespace when a path source exists", () => {
    const sources = new Map([
      [
        "amkisko",
        source("amkisko", "git", "git+https://example.com/prayers.git"),
      ],
      ["local", source("local", "path", "prayers")],
    ]);
    assert.equal(
      impliedSourceName(packageEntry("amkisko/rules"), sources),
      "amkisko",
    );
  });
});
