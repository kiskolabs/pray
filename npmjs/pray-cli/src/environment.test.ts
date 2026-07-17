import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  packageMatchesEnvironment,
  shouldRenderPackage,
  validateEnvironment,
} from "./environment.js";
import { PrayError } from "./errors.js";
import type { Manifest, ManifestPackage } from "./manifest/types.js";

function packageWithGroups(groups: string[]): ManifestPackage {
  return {
    name: "sample/base",
    constraint: "*",
    exports: [],
    targets: [],
    features: [],
    groups,
    optional: false,
  };
}

describe("environment", () => {
  it("keeps ungrouped packages for every environment selection", () => {
    const packageEntry = packageWithGroups([]);
    assert.equal(shouldRenderPackage(packageEntry, undefined), true);
    assert.equal(shouldRenderPackage(packageEntry, "development"), true);
  });

  it("renders grouped packages only for selected environments", () => {
    const packageEntry = packageWithGroups(["development", "test"]);
    assert.equal(
      packageMatchesEnvironment(packageEntry.groups, undefined),
      false,
    );
    assert.equal(
      packageMatchesEnvironment(packageEntry.groups, "development"),
      true,
    );
    assert.equal(
      packageMatchesEnvironment(packageEntry.groups, "production"),
      false,
    );
  });

  it("rejects unknown environments when groups exist", () => {
    const manifest: Manifest = {
      prayfileVersion: "1",
      sources: [],
      targets: [],
      packages: [packageWithGroups(["development"])],
      local: [],
      render: {
        mode: "managed",
        conflict: "fail",
        churn: "minimal",
        header: true,
        sectionMarkers: true,
        lineEndings: "lf",
      },
    };
    assert.throws(
      () => validateEnvironment(manifest, "production"),
      (error: unknown) =>
        error instanceof PrayError &&
        error.kind === "resolution" &&
        error.message.includes("unknown environment production"),
    );
  });
});
