import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import {
  classifyFormatHints,
  formatRecommended,
  type PackageFormatHint,
  recommendManifest,
  usesDestinationDsl,
} from "./manifest/format-manifest.js";
import { parseManifest } from "./manifest/index.js";
import { resolveProject } from "./resolve/project.js";

function writePackage(
  root: string,
  directory: string,
  packageName: string,
  exportName: string,
  exportKind: string,
  exportPath: string,
  body: string,
  defaultPath?: string,
): void {
  const packageRoot = join(root, "packages", directory);
  mkdirSync(join(packageRoot, "exports"), { recursive: true });
  const defaultPathLiteral = defaultPath
    ? `,\n      default_path: "${defaultPath}"`
    : "";
  writeFileSync(
    join(packageRoot, `${directory}.prayspec`),
    `
Package::Specification.new do |spec|
  spec.name = "${packageName}"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = ["${exportPath}"]
  spec.exports = {
    "${exportName}" => {
      type: "${exportKind}",
      path: "${exportPath}",
      summary: "${exportName}"${defaultPathLiteral}
    }
  }
end
`,
  );
  writeFileSync(join(packageRoot, exportPath), body);
}

describe("format manifest", () => {
  it("formats a legacy Prayfile into the recommended destination DSL", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-format-legacy-"));
    mkdirSync(join(root, ".agents"), { recursive: true });
    writePackage(
      root,
      "rules",
      "sample/rules",
      "rules",
      "fragment",
      "exports/rules.md",
      "Rules\n",
    );
    mkdirSync(join(root, "packages/audit/skills/audit"), { recursive: true });
    writeFileSync(
      join(root, "packages/audit/skills/audit/SKILL.md"),
      "# Audit\n",
    );
    writeFileSync(
      join(root, "packages/audit/audit.prayspec"),
      `
Package::Specification.new do |spec|
  spec.name = "sample/audit"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = ["skills/audit/SKILL.md"]
  spec.exports = {
    "audit" => {
      type: "skill",
      path: "skills/audit",
      summary: "audit"
    }
  }
end
`,
    );
    writePackage(
      root,
      "security",
      "sample/security",
      "security",
      "file",
      "exports/SECURITY.md",
      "# Security\n",
      "SECURITY.md",
    );
    writeFileSync(join(root, ".agents/project.md"), "Local\n");

    const original = `
prayfile "1"
target :tool_a do
  output "AGENTS.md"
  skills ".agents/skills"
end
agent "sample/rules", "~> 1.0", path: "packages/rules"
agent "sample/audit", "~> 1.0", path: "packages/audit"
agent "sample/security", "~> 1.0", path: "packages/security"
local ".agents/project.md", position: :before
`;
    writeFileSync(join(root, "Prayfile"), original);

    const project = await resolveProject(join(root, "Prayfile"));
    const hints = classifyFormatHints(project);
    const manifest = parseManifest(original);
    assert.equal(usesDestinationDsl(manifest), false);

    const formatted = formatRecommended(manifest, hints);
    assert.match(formatted, /compose "AGENTS\.md" do/);
    assert.match(formatted, /pray "\.agents\/project\.md"/);
    assert.match(formatted, /pray "sample\/rules"/);
    assert.match(formatted, /tree "\.agents\/skills" do/);
    assert.match(formatted, /pray "sample\/audit"/);
    assert.match(formatted, /file: "SECURITY\.md"/);
    assert.doesNotMatch(formatted, /target :tool_a/);
    assert.doesNotMatch(formatted, /agent "/);

    const reparsed = parseManifest(formatted);
    assert.equal(usesDestinationDsl(reparsed), true);
    assert.equal(reparsed.targets[0]?.mode, "compose");
    assert.equal(reparsed.targets[1]?.mode, "tree");
    const security = reparsed.packages.find(
      (entry) => entry.name === "sample/security",
    );
    assert.equal(security?.file, "SECURITY.md");
    assert.ok(security?.roles?.includes("file"));

    const again = formatRecommended(reparsed, hints);
    assert.equal(again, formatted);
  });

  it("formats a legacy Prayfile that already has file: bindings", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-format-hybrid-"));
    mkdirSync(join(root, ".agents"), { recursive: true });
    writePackage(
      root,
      "rules",
      "sample/rules",
      "rules",
      "fragment",
      "exports/rules.md",
      "Rules\n",
    );
    mkdirSync(join(root, "packages/audit/skills/audit"), { recursive: true });
    writeFileSync(
      join(root, "packages/audit/skills/audit/SKILL.md"),
      "# Audit\n",
    );
    writeFileSync(
      join(root, "packages/audit/audit.prayspec"),
      `
Package::Specification.new do |spec|
  spec.name = "sample/audit"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = ["skills/audit/SKILL.md"]
  spec.exports = {
    "audit" => {
      type: "skill",
      path: "skills/audit",
      summary: "audit"
    }
  }
end
`,
    );
    writePackage(
      root,
      "security",
      "sample/security",
      "security",
      "file",
      "exports/SECURITY.md",
      "# Security\n",
      "SECURITY.md",
    );
    writeFileSync(join(root, ".agents/project.md"), "Local\n");

    const original = `
prayfile "1"
target :tool_a do
  output "AGENTS.md"
  skills ".agents/skills"
end
agent "sample/rules", "~> 1.0", path: "packages/rules"
agent "sample/audit", "~> 1.0", path: "packages/audit"
pray "sample/security", "~> 1.0", path: "packages/security", file: "SECURITY.md"
local ".agents/project.md", position: :before
`;
    writeFileSync(join(root, "Prayfile"), original);

    const project = await resolveProject(join(root, "Prayfile"));
    const hints = classifyFormatHints(project);
    const manifest = parseManifest(original);
    assert.equal(usesDestinationDsl(manifest), true);

    const formatted = formatRecommended(manifest, hints);
    assert.match(formatted, /compose "AGENTS\.md" do/);
    assert.match(formatted, /tree "\.agents\/skills" do/);
    assert.match(formatted, /file: "SECURITY\.md"/);
    assert.doesNotMatch(formatted, /target :tool_a/);
  });

  it("omits source keyword when namespace matches a source handle", () => {
    const manifest = parseManifest(`
prayfile "1"
source "amkisko", path: "packages/amkisko"
source "other", path: "packages/other"
compose "AGENTS.md" do
  pray "amkisko/rules", "~> 1.0", source: "amkisko"
  pray "other/notes", "~> 1.0", source: "other"
end
`);
    const formatted = formatRecommended(manifest, new Map());
    assert.match(formatted, /pray "amkisko\/rules", "~> 1\.0"/);
    assert.match(formatted, /pray "other\/notes", "~> 1\.0"/);
    assert.doesNotMatch(formatted, /source: "amkisko"/);
    assert.doesNotMatch(formatted, /source: "other"/);
  });

  it("formats an existing destination DSL manifest idempotently", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-format-dsl-"));
    mkdirSync(join(root, ".agents"), { recursive: true });
    writePackage(
      root,
      "rules",
      "sample/rules",
      "rules",
      "fragment",
      "exports/rules.md",
      "Rules\n",
    );
    writeFileSync(join(root, ".agents/project.md"), "Local\n");

    const original = `
prayfile "1"
compose "AGENTS.md" do
  pray ".agents/project.md"
  pray "sample/rules", "~> 1.0", path: "packages/rules"
end
`;
    writeFileSync(join(root, "Prayfile"), original);

    const project = await resolveProject(join(root, "Prayfile"));
    const hints = classifyFormatHints(project);
    const manifest = parseManifest(original);
    const formatted = formatRecommended(manifest, hints);
    const again = formatRecommended(parseManifest(formatted), hints);
    assert.equal(formatted, again);
    assert.match(formatted, /compose "AGENTS\.md" do/);
  });

  it("classifies package roles from hints when recommending a manifest", () => {
    const manifest = parseManifest(`
prayfile "1"
target :tool_a do
  output "AGENTS.md"
  skills ".agents/skills"
end
agent "sample/rules", "~> 1.0", path: "packages/rules"
agent "sample/audit", "~> 1.0", path: "packages/audit"
`);
    const hints = new Map<string, PackageFormatHint>([
      ["sample/rules", { roles: ["fragment"] }],
      ["sample/audit", { roles: ["folder"] }],
    ]);

    const recommended = recommendManifest(manifest, hints);
    assert.equal(recommended.targets.length, 2);
    assert.ok(
      recommended.targets[0]?.entries?.some(
        (entry) => entry.kind === "package" && entry.name === "sample/rules",
      ),
    );
    assert.ok(
      recommended.targets[1]?.entries?.some(
        (entry) => entry.kind === "package" && entry.name === "sample/audit",
      ),
    );
  });
});
