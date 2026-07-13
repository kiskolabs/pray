import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { parseManifest } from "./manifest/index.js";
import { parsePackageSpec } from "./package-spec/index.js";
import { PrayError } from "./errors.js";

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
