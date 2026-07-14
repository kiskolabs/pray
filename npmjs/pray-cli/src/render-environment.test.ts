import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { renderProject } from "./render/project.js";
import type { ManifestPackage } from "./manifest/types.js";
import type { PackageSpec } from "./package-spec/types.js";
import type { ResolvedPackage, ResolvedProject } from "./resolve/types.js";

function resolvedPackage(
  name: string,
  groups: string[],
  exportBody: string,
): ResolvedPackage {
  const declaration: ManifestPackage = {
    name,
    constraint: "*",
    exports: ["guidance"],
    targets: [],
    features: [],
    groups,
    optional: false,
  };
  const spec: PackageSpec = {
    name,
    version: "1.0.0",
    summary: "summary",
    authors: [],
    files: [],
    exports: new Map([
      [
        "guidance",
        {
          kind: "fragment",
          path: "exports/guidance.md",
          summary: "guidance",
        },
      ],
    ]),
    skills: new Map(),
    templates: new Map(),
    adapters: new Map(),
    targets: [],
    dependencies: [],
    metadata: new Map(),
  };
  return {
    declaration,
    root: "/tmp/package",
    spec,
    treeHash: "sha256:tree",
    artifactHash: "sha256:artifact",
    artifact: "path:/tmp/package",
    selectedExports: ["guidance"],
    sourceChecksum: "sha256:source",
    exportBodies: new Map([["guidance", exportBody]]),
    skillFiles: new Map(),
  };
}

function projectWithEnvironment(
  environment: string | undefined,
  packages: ResolvedPackage[],
): ResolvedProject {
  return {
    manifestPath: "/tmp/project/Prayfile",
    projectRoot: "/tmp/project",
    manifestHash: "sha256:manifest",
    manifest: {
      prayfileVersion: "1",
      sources: [],
      targets: [
        {
          name: "tool_a",
          outputs: ["AGENTS.md"],
          skills: [],
          commands: [],
          rules: [],
        },
      ],
      packages: packages.map((packageEntry) => packageEntry.declaration),
      local: [],
      render: {
        mode: "managed",
        conflict: "fail",
        churn: "minimal",
        header: false,
        sectionMarkers: true,
        lineEndings: "lf",
      },
    },
    packages,
    localFiles: [],
    sourceRevisions: new Map(),
    sourceHostKeys: new Map(),
    ...(environment ? { environment } : {}),
  };
}

describe("render environment filtering", () => {
  it("renders only packages matching the selected environment", () => {
    const shared = resolvedPackage("sample/shared", [], "shared guidance");
    const development = resolvedPackage(
      "sample/development",
      ["development"],
      "development guidance",
    );
    const rendered = renderProject(
      projectWithEnvironment("development", [shared, development]),
    );
    const content = rendered[0]?.content ?? "";
    assert.match(content, /shared guidance/);
    assert.match(content, /development guidance/);
    assert.equal(rendered[0]?.managedSpans.length, 2);
  });

  it("renders only ungrouped packages when no environment is selected", () => {
    const shared = resolvedPackage("sample/shared", [], "shared guidance");
    const development = resolvedPackage(
      "sample/development",
      ["development"],
      "development guidance",
    );
    const rendered = renderProject(
      projectWithEnvironment(undefined, [shared, development]),
    );
    const content = rendered[0]?.content ?? "";
    assert.match(content, /shared guidance/);
    assert.doesNotMatch(content, /development guidance/);
    assert.equal(rendered[0]?.managedSpans.length, 1);
  });
});
