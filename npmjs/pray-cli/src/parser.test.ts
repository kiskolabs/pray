import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { PrayError } from "./errors.js";
import { parseManifest } from "./manifest/index.js";
import { parsePackageSpec } from "./package-spec/index.js";

describe("parser", () => {
  it("parses minimal manifest example", () => {
    const manifest = parseManifest(`
prayfile "1"
source "default", "https://agents.example.com"
target :tool_a do
  output "INSTRUCTIONS.md"
  skills ".agents/skills"
end
agent "sample/base", "~> 1.4",
  exports: ["testing-basics", "security-basics"]
local ".agents/project.md"
render mode: :managed,
  conflict: :fail,
  churn: :minimal
`);

    assert.equal(manifest.prayfileVersion, "1");
    assert.equal(manifest.sources[0]?.name, "default");
    assert.equal(manifest.targets[0]?.name, "tool_a");
    assert.deepEqual(manifest.targets[0]?.outputs, ["INSTRUCTIONS.md"]);
    assert.equal(manifest.packages[0]?.name, "sample/base");
    assert.equal(manifest.local[0]?.path, ".agents/project.md");
    assert.equal(manifest.render.mode, "managed");
  });

  it("parses group blocks as render selectors", () => {
    const manifest = parseManifest(`
prayfile "1"
group :development, :test do
  agent "sample/dev", "*"
end
agent "sample/shared", "*"
`);

    assert.deepEqual(manifest.packages[0]?.groups, ["development", "test"]);
    assert.equal(manifest.packages[0]?.name, "sample/dev");
    assert.deepEqual(manifest.packages[1]?.groups, []);
    assert.equal(manifest.packages[1]?.name, "sample/shared");
  });

  it("rejects nested group blocks", () => {
    assert.throws(
      () =>
        parseManifest(`
prayfile "1"
group :development do
  group :test do
    agent "sample/dev", "*"
  end
end
`),
      (error: unknown) =>
        error instanceof PrayError &&
        error.kind === "parse" &&
        error.message.includes(
          "group blocks only support agent, package, or pray declarations",
        ),
    );
  });

  it("rejects non-package statements inside group blocks", () => {
    assert.throws(
      () =>
        parseManifest(`
prayfile "1"
group :development do
  source "default", "https://agents.example.com"
end
`),
      (error: unknown) =>
        error instanceof PrayError &&
        error.kind === "parse" &&
        error.message.includes(
          "group blocks only support agent, package, or pray declarations",
        ),
    );
  });

  it("accepts pray declarations inside group blocks", () => {
    const manifest = parseManifest(`
prayfile "1"
group :development do
  pray "sample/dev", "*"
end
`);

    assert.deepEqual(manifest.packages[0]?.groups, ["development"]);
    assert.equal(manifest.packages[0]?.name, "sample/dev");
  });

  it("parses minimal package spec example", () => {
    const packageSpec = parsePackageSpec(`
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.4.3"
  spec.summary = "shared guidance"
  spec.files = ["README.md", "exports/testing-basics.md"]
  spec.exports = {
    "testing-basics" => {
      type: "fragment",
      path: "exports/testing-basics.md",
      summary: "Testing guidance"
    }
  }
  spec.add_dependency "sample/common", "~> 1.0"
end
`);

    assert.equal(packageSpec.name, "sample/base");
    assert.equal(packageSpec.version, "1.4.3");
    assert.deepEqual(packageSpec.files, [
      "README.md",
      "exports/testing-basics.md",
    ]);
    assert.equal(
      packageSpec.exports.get("testing-basics")?.path,
      "exports/testing-basics.md",
    );
    assert.equal(packageSpec.dependencies[0]?.name, "sample/common");
  });

  it("parses compose blocks with pray and local entries", () => {
    const manifest = parseManifest(`
prayfile "1"
compose "AGENTS.md" do
  pray ".agents/project.md"
  pray "sample/rules", "~> 1.0", path: "packages/rules"
end
`);

    assert.equal(manifest.targets[0]?.mode, "compose");
    assert.equal(manifest.targets[0]?.scoped, true);
    assert.deepEqual(manifest.targets[0]?.outputs, ["AGENTS.md"]);
    assert.deepEqual(manifest.targets[0]?.entries, [
      { kind: "local", path: ".agents/project.md" },
      { kind: "package", name: "sample/rules" },
    ]);
    assert.equal(manifest.local[0]?.bound, true);
    assert.equal(manifest.packages[0]?.bound, true);
  });

  it("parses tree blocks scoping packages to a provisioned folder", () => {
    const manifest = parseManifest(`
prayfile "1"
tree ".agents/skills" do
  pray "sample/audit", "~> 1.0", path: "packages/audit"
end
`);

    assert.equal(manifest.targets[0]?.mode, "tree");
    assert.equal(manifest.targets[0]?.scoped, true);
    assert.deepEqual(manifest.targets[0]?.skills, [".agents/skills"]);
    assert.equal(manifest.packages[0]?.bound, true);
  });

  it("parses file: on a pray declaration for exact bindings", () => {
    const manifest = parseManifest(`
prayfile "1"
pray "sample/security", "~> 1.0", path: "packages/security", file: "SECURITY.md"
`);

    assert.equal(manifest.packages[0]?.file, "SECURITY.md");
    assert.deepEqual(manifest.packages[0]?.roles, ["file"]);
  });

  it("parses a file block with a single pray declaration", () => {
    const manifest = parseManifest(`
prayfile "1"
file "SECURITY.md" do
  pray "sample/security", "~> 1.0", path: "packages/security"
end
`);

    assert.equal(manifest.packages[0]?.file, "SECURITY.md");
  });

  it("rejects a file block without a pray declaration", () => {
    assert.throws(
      () =>
        parseManifest(`
prayfile "1"
file "SECURITY.md" do
end
`),
      (error: unknown) =>
        error instanceof PrayError &&
        error.kind === "parse" &&
        error.message.includes("requires a pray package declaration"),
    );
  });

  it("rejects manifest without prayfile version", () => {
    assert.throws(
      () =>
        parseManifest(`
target :tool_a do
  output "INSTRUCTIONS.md"
end
`),
      (error: unknown) =>
        error instanceof PrayError &&
        error.kind === "manifest" &&
        error.message.includes("missing prayfile version"),
    );
  });
});
