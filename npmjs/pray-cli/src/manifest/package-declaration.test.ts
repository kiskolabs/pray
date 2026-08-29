import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { replacePackageDeclaration } from "./package-declaration.js";
import type { ManifestPackage } from "./types.js";

function samplePackage(constraint: string): ManifestPackage {
  return {
    name: "sample/base",
    constraint,
    exports: [],
    targets: [],
    features: [],
    groups: [],
    optional: false,
  };
}

describe("replacePackageDeclaration", () => {
  it("rewrites every matching line and keeps indent and extra keywords", () => {
    const text = `
prayfile "1"
compose "AGENTS.md" do
  pray "sample/base", "~> 1.0"
end
tree ".agents/skills" do
  pray "sample/base", "~> 1.0", export: "testing-basics"
end
`;
    const updated = replacePackageDeclaration(text, samplePackage("~> 1.1"));
    assert.match(updated, / {2}pray "sample\/base", "~> 1.1"/);
    assert.match(
      updated,
      / {2}pray "sample\/base", "~> 1.1", export: "testing-basics"/,
    );
    assert.equal(updated.includes("~> 1.0"), false);
  });

  it("inserts a constraint before keyword arguments", () => {
    const text = 'pray "sample/base", path: "packages/base"\n';
    const updated = replacePackageDeclaration(text, samplePackage("~> 1.1"));
    assert.equal(
      updated,
      'pray "sample/base", "~> 1.1", path: "packages/base"\n',
    );
  });

  it("appends a constraint when the line has only a name", () => {
    const text = 'pray "sample/base"\n';
    const updated = replacePackageDeclaration(text, samplePackage("~> 1.1"));
    assert.equal(updated, 'pray "sample/base", "~> 1.1"\n');
  });
});
