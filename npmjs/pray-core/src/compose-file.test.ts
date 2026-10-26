import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { exportKindMatchesRole } from "./manifest/destination.js";
import type { ExportRole } from "./manifest/types.js";
import { parsePackageSpec } from "./package-spec/index.js";
import { renderProject, writeRenderedTargets } from "./render/project.js";
import { writePackage } from "./render-test-package.js";
import { resolveProject } from "./resolve/project.js";

async function renderRoot(root: string) {
  return renderProject(await resolveProject(join(root, "Prayfile")));
}

describe("compose file export", () => {
  it("inlines a utf-8 file export into compose as a marked span", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-compose-file-"));
    writePackage(
      root,
      "community",
      "sample/community",
      "contributing",
      "file",
      "exports/CONTRIBUTING.md",
      "Be kind.\n",
    );
    writeFileSync(
      join(root, "Prayfile"),
      `
prayfile "1"
compose "CONTRIBUTING.md" do
  pray "sample/community", "~> 1.0", path: "packages/community"
end
`,
    );
    const rendered = await renderRoot(root);
    assert.match(rendered[0]?.content ?? "", /<!-- pray:/);
    assert.match(rendered[0]?.content ?? "", /Be kind/);
    assert.doesNotMatch(rendered[0]?.content ?? "", /# Agent context/);
  });

  it("inlines an empty utf-8 file export", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-compose-empty-file-"));
    writePackage(
      root,
      "empty",
      "sample/empty",
      "empty",
      "file",
      "exports/EMPTY.md",
      "",
    );
    writeFileSync(
      join(root, "Prayfile"),
      `
prayfile "1"
compose "EMPTY.md" do
  pray "sample/empty", "~> 1.0", path: "packages/empty"
end
`,
    );

    const rendered = await renderRoot(root);
    assert.match(rendered[0]?.content ?? "", /<!-- pray:/);
  });

  it("preserves unmanaged text around an existing managed span", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-compose-preserve-local-"));
    writePackage(
      root,
      "rules",
      "sample/rules",
      "rules",
      "fragment",
      "exports/rules.md",
      "Old managed text\n",
    );
    writeFileSync(
      join(root, "Prayfile"),
      `
prayfile "1"
compose "AGENTS.md" do
  pray "sample/rules", "~> 1.0", path: "packages/rules"
end
`,
    );
    let project = await resolveProject(join(root, "Prayfile"));
    writeRenderedTargets(project, renderProject(project));
    const destination = join(root, "AGENTS.md");
    writeFileSync(
      destination,
      `${readFileSync(destination, "utf8")}\nLocal text must survive.\n`,
    );
    writeFileSync(
      join(root, "packages/rules/exports/rules.md"),
      "New managed text\n",
    );

    project = await resolveProject(join(root, "Prayfile"));
    writeRenderedTargets(project, renderProject(project));
    const content = readFileSync(destination, "utf8");
    assert.match(content, /Local text must survive\./);
    assert.match(content, /New managed text/);
    assert.doesNotMatch(content, /Old managed text/);
  });

  it("keeps exclusive file destinations unmarked", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-exclusive-file-"));
    writePackage(
      root,
      "community",
      "sample/community",
      "contributing",
      "file",
      "exports/CONTRIBUTING.md",
      "Be kind.\n",
    );
    writeFileSync(
      join(root, "Prayfile"),
      `
prayfile "1"
pray "sample/community", "~> 1.0", path: "packages/community", file: "CONTRIBUTING.md"
`,
    );
    const project = await resolveProject(join(root, "Prayfile"));
    const rendered = renderProject(project);
    assert.equal(rendered.length, 0);
    writeRenderedTargets(project, rendered);
    const dest = readFileSync(join(root, "CONTRIBUTING.md"), "utf8");
    assert.equal(dest, "Be kind.\n");
    assert.doesNotMatch(dest, /<!-- pray:/);
  });

  it("prefers a fragment when a file export also exists", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-compose-prefer-fragment-"));
    const packageRoot = join(root, "packages/mixed");
    mkdirSync(join(packageRoot, "exports"), { recursive: true });
    writeFileSync(
      join(packageRoot, "mixed.prayspec"),
      `
Package::Specification.new do |spec|
  spec.name = "sample/mixed"
  spec.version = "1.0.0"
  spec.summary = "fixture"
  spec.files = ["exports/notes.md", "exports/CONTRIBUTING.md"]
  spec.exports = {
    "notes" => { type: "fragment", path: "exports/notes.md" },
    "contributing" => { type: "file", path: "exports/CONTRIBUTING.md" }
  }
end
`,
    );
    writeFileSync(join(packageRoot, "exports/notes.md"), "Fragment notes\n");
    writeFileSync(
      join(packageRoot, "exports/CONTRIBUTING.md"),
      "File contributing\n",
    );
    writeFileSync(
      join(root, "Prayfile"),
      `
prayfile "1"
compose "AGENTS.md" do
  pray "sample/mixed", "~> 1.0", path: "packages/mixed"
end
`,
    );
    const rendered = await renderRoot(root);
    assert.match(rendered[0]?.content ?? "", /Fragment notes/);
    assert.doesNotMatch(rendered[0]?.content ?? "", /File contributing/);
    assert.match(rendered[0]?.content ?? "", /# Agent context/);
    assert.match(rendered[0]?.content ?? "", /\.agents\//);
  });

  it("fails compose of a binary file export", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-compose-binary-"));
    writePackage(
      root,
      "blob",
      "sample/blob",
      "icon",
      "file",
      "exports/icon.md",
      new Uint8Array([0xff, 0xfe, 0x00]),
    );
    writeFileSync(
      join(root, "Prayfile"),
      `
prayfile "1"
compose "ICON.md" do
  pray "sample/blob", "~> 1.0", path: "packages/blob"
end
`,
    );
    const project = await resolveProject(join(root, "Prayfile"));
    assert.throws(
      () => renderProject(project),
      (error: unknown) =>
        error instanceof Error &&
        (error.message.includes("binary") || error.message.includes("utf-8")),
    );
  });

  it("fails closed on compose of JSON", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-compose-json-"));
    writePackage(
      root,
      "rules",
      "sample/rules",
      "rules",
      "fragment",
      "exports/rules.md",
      "Keep it small.\n",
    );
    writeFileSync(
      join(root, "Prayfile"),
      `
prayfile "1"
compose "config.json" do
  pray "sample/rules", "~> 1.0", path: "packages/rules"
end
`,
    );
    const project = await resolveProject(join(root, "Prayfile"));
    assert.throws(
      () => renderProject(project),
      (error: unknown) =>
        error instanceof Error &&
        error.message.includes("JSON") &&
        error.message.includes('file: "config.json"'),
    );
  });

  it("fails closed on compose of an unknown type", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-compose-unknown-"));
    writePackage(
      root,
      "rules",
      "sample/rules",
      "rules",
      "fragment",
      "exports/rules.md",
      "Keep it small.\n",
    );
    writeFileSync(
      join(root, "Prayfile"),
      `
prayfile "1"
compose ".zshrc" do
  pray "sample/rules", "~> 1.0", path: "packages/rules"
end
`,
    );
    const project = await resolveProject(join(root, "Prayfile"));
    assert.throws(
      () => renderProject(project),
      (error: unknown) =>
        error instanceof Error && error.message.includes('file: ".zshrc"'),
    );
  });

  it("suppresses the Agent banner when compose sets header: false", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-compose-header-off-"));
    writePackage(
      root,
      "rules",
      "sample/rules",
      "rules",
      "fragment",
      "exports/rules.md",
      "Keep it small.\n",
    );
    writeFileSync(
      join(root, "Prayfile"),
      `
prayfile "1"
compose "AGENTS.md", header: false do
  pray "sample/rules", "~> 1.0", path: "packages/rules"
end
`,
    );
    const rendered = await renderRoot(root);
    assert.doesNotMatch(rendered[0]?.content ?? "", /# Agent context/);
  });

  it("omits .agents from a forced banner on NOTES.md", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-compose-header-on-"));
    writePackage(
      root,
      "rules",
      "sample/rules",
      "rules",
      "fragment",
      "exports/rules.md",
      "Keep it small.\n",
    );
    writeFileSync(
      join(root, "Prayfile"),
      `
prayfile "1"
compose "NOTES.md", header: true do
  pray "sample/rules", "~> 1.0", path: "packages/rules"
end
`,
    );
    const rendered = await renderRoot(root);
    assert.match(rendered[0]?.content ?? "", /# Agent context/);
    assert.doesNotMatch(rendered[0]?.content ?? "", /\.agents\//);
  });

  it("does not match unused export kinds to a destination role", () => {
    const roles: ExportRole[] = ["fragment", "folder", "file"];
    for (const kind of ["template", "command", "rule", "asset", "bundle"]) {
      for (const role of roles) {
        assert.equal(exportKindMatchesRole(kind, role), false);
      }
    }
  });

  it("parses spec.adapters and does not load them", () => {
    const spec = parsePackageSpec(`
Package::Specification.new do |spec|
  spec.name = "sample/with-adapters"
  spec.version = "1.0.0"
  spec.files = ["exports/a.md"]
  spec.exports = { "a" => { type: "fragment", path: "exports/a.md" } }
  spec.adapters = { "tool_a" => "adapters/tool_a.toml" }
end
`);
    assert.equal(spec.adapters.get("tool_a"), "adapters/tool_a.toml");
  });
});
