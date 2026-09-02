import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { buildLockfile } from "./lockfile/index.js";
import { renderProject, writeRenderedTargets } from "./render/project.js";
import { plannedProvisionedFiles } from "./render/provisioned.js";
import { writePackage } from "./render-test-package.js";
import { resolveProject } from "./resolve/project.js";
import { verifyProject } from "./verify/project.js";

describe("destination render", () => {
  it("still fans out fragments and skills for the legacy shape", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-legacy-fanout-"));
    mkdirSync(join(root, ".agents"), { recursive: true });
    writePackage(
      root,
      "rules",
      "sample/rules",
      "rules",
      "fragment",
      "exports/rules.md",
      "Legacy rules\n",
    );
    mkdirSync(join(root, "packages/audit/skills/audit"), { recursive: true });
    writeFileSync(
      join(root, "packages/audit/skills/audit/SKILL.md"),
      "# Audit skill\n",
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
    writeFileSync(join(root, ".agents/project.md"), "Local note\n");
    writeFileSync(
      join(root, "Prayfile"),
      `
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
  skills ".agents/skills"
end
agent "sample/rules", "~> 1.0", path: "packages/rules"
agent "sample/audit", "~> 1.0", path: "packages/audit"
local ".agents/project.md"
`,
    );

    const project = await resolveProject(join(root, "Prayfile"));
    const rendered = renderProject(project);
    assert.equal(rendered.length, 1);
    assert.match(rendered[0]?.content ?? "", /Legacy rules/);
    assert.match(rendered[0]?.content ?? "", /Local note/);
    assert.match(rendered[0]?.content ?? "", /## Shared instructions/);

    const planned = plannedProvisionedFiles(project);
    assert.ok(
      planned.some((file) =>
        file.path.endsWith(".agents/skills/audit/SKILL.md"),
      ),
    );
  });

  it("isolates compose, tree, and file bindings", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-destination-dsl-"));
    mkdirSync(join(root, ".agents"), { recursive: true });
    writePackage(
      root,
      "rules",
      "sample/rules",
      "rules",
      "fragment",
      "exports/rules.md",
      "Compose rules\n",
    );
    writePackage(
      root,
      "unbound",
      "sample/unbound",
      "unbound",
      "fragment",
      "exports/unbound.md",
      "Should not appear\n",
    );
    mkdirSync(join(root, "packages/audit/skills/audit"), { recursive: true });
    writeFileSync(
      join(root, "packages/audit/skills/audit/SKILL.md"),
      "# Audit skill\n",
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
    mkdirSync(join(root, "packages/security/exports"), { recursive: true });
    writeFileSync(
      join(root, "packages/security/exports/SECURITY.md"),
      "# Security Policy\n\nEmail: ((pray:security_email))\n",
    );
    writeFileSync(
      join(root, "packages/security/security.prayspec"),
      `
Package::Specification.new do |spec|
  spec.name = "sample/security"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = ["exports/SECURITY.md"]
  spec.exports = {
    "security" => {
      type: "file",
      path: "exports/SECURITY.md",
      default_path: "SECURITY.md"
    }
  }
end
`,
    );
    writeFileSync(join(root, ".agents/project.md"), "Local first\n");
    writeFileSync(
      join(root, "Prayfile"),
      `
prayfile "1"
pray do
  security_email "security@example.com"
end
compose "AGENTS.md" do
  pray ".agents/project.md"
  pray "sample/rules", "~> 1.0", path: "packages/rules"
end
tree ".agents/skills" do
  pray "sample/audit", "~> 1.0", path: "packages/audit"
end
pray "sample/security", "~> 1.0", path: "packages/security", file: "SECURITY.md"
agent "sample/unbound", "~> 1.0", path: "packages/unbound"
`,
    );

    const project = await resolveProject(join(root, "Prayfile"));
    const rendered = renderProject(project);
    assert.equal(rendered.length, 1);
    const content = rendered[0]?.content ?? "";
    assert.match(content, /Local first/);
    assert.match(content, /Compose rules/);
    assert.doesNotMatch(content, /Should not appear/);
    assert.doesNotMatch(content, /## Shared instructions/);

    const planned = plannedProvisionedFiles(project);
    assert.ok(planned.some((file) => file.path === "SECURITY.md"));
    assert.ok(
      planned.some((file) =>
        file.path.endsWith(".agents/skills/audit/SKILL.md"),
      ),
    );
    assert.ok(
      !planned.some((file) => file.path.includes("security/SECURITY.md")),
    );

    writeRenderedTargets(project, rendered);
    const security = readFileSync(join(root, "SECURITY.md"), "utf8");
    assert.equal(
      security,
      "# Security Policy\n\nEmail: security@example.com\n",
    );

    const lockfile = buildLockfile({
      manifestHash: project.manifestHash,
      projectRoot: project.projectRoot,
      manifestSources: project.manifest.sources,
      manifestTargets: project.manifest.targets,
      rendered,
      packages: project.packages,
      project,
    });
    const report = verifyProject(project, lockfile, true);
    assert.equal(report.findings.length, 0);
  });

  it("limits the provisioned tree with a folder export only filter", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-folder-only-"));
    mkdirSync(join(root, "packages/templates/templates"), { recursive: true });
    writeFileSync(
      join(root, "packages/templates/templates/issue.md"),
      "issue\n",
    );
    writeFileSync(join(root, "packages/templates/templates/pr.md"), "pr\n");
    writeFileSync(
      join(root, "packages/templates/templates/draft.md"),
      "draft\n",
    );
    writeFileSync(
      join(root, "packages/templates/templates.prayspec"),
      `
Package::Specification.new do |spec|
  spec.name = "sample/templates"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = ["templates/issue.md", "templates/pr.md", "templates/draft.md"]
  spec.exports = {
    "templates" => {
      type: "folder",
      path: "templates",
      only: ["issue.md", "pr.md"]
    }
  }
end
`,
    );
    writeFileSync(
      join(root, "Prayfile"),
      `
prayfile "1"
tree ".agents/templates" do
  pray "sample/templates", "~> 1.0", path: "packages/templates"
end
`,
    );

    const project = await resolveProject(join(root, "Prayfile"));
    const planned = plannedProvisionedFiles(project);
    assert.ok(planned.some((file) => file.path.endsWith("issue.md")));
    assert.ok(planned.some((file) => file.path.endsWith("pr.md")));
    assert.ok(!planned.some((file) => file.path.endsWith("draft.md")));
  });

  it("parses singular export and alias keywords for resolution", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-export-alias-"));
    writePackage(
      root,
      "rules",
      "sample/rules",
      "rules",
      "fragment",
      "exports/rules.md",
      "Alias rules\n",
    );
    writeFileSync(
      join(root, "Prayfile"),
      `
prayfile "1"
compose "AGENTS.md" do
  include "sample/rules", "~> 1.0", path: "packages/rules", export: "rules"
end
`,
    );

    const project = await resolveProject(join(root, "Prayfile"));
    assert.deepEqual(project.packages[0]?.selectedExports, ["rules"]);
    const rendered = renderProject(project);
    assert.match(rendered[0]?.content ?? "", /Alias rules/);
  });
});
