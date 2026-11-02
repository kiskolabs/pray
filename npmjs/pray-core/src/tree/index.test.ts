import assert from "node:assert/strict";
import { describe, it } from "node:test";
import type { PackageSpec } from "../package-spec/types.js";
import type { ResolvedPackage, ResolvedProject } from "../resolve/types.js";
import { renderDependencyTree } from "./index.js";

function packageEntry(
  name: string,
  dependencies: PackageSpec["dependencies"],
): ResolvedPackage {
  return {
    declaration: {
      name,
      constraint: "*",
      exports: [],
      targets: [],
      features: [],
      groups: [],
      optional: false,
    },
    root: ".",
    spec: {
      name,
      version: "1.0.0",
      authors: [],
      maintainers: [],
      files: [],
      exports: new Map(),
      skills: new Map(),
      templates: new Map(),
      adapters: new Map(),
      targets: [],
      related: [],
      dependencies,
      metadata: new Map(),
    },
    treeHash: "sha256:test",
    artifactHash: "sha256:test",
    artifact: "path:.",
    selectedExports: [],
    sourceChecksum: "sha256:test",
    exportBodies: new Map(),
    skillFiles: new Map(),
  };
}

describe("dependency tree", () => {
  it("omits optional referrals", () => {
    const base = packageEntry("sample/base", [
      { name: "sample/common", constraint: "~> 1.0", optional: false },
      { name: "sample/peer", constraint: "~> 1.0", optional: true },
    ]);
    const common = packageEntry("sample/common", []);
    const peer = packageEntry("sample/peer", []);
    const project = { packages: [base, common, peer] } as ResolvedProject;

    const lines = renderDependencyTree(project);

    assert.ok(lines.includes("  sample/common 1.0.0"));
    assert.ok(!lines.includes("  sample/peer 1.0.0"));
  });
});
