import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { parsePackageSpec } from "../package-spec/index.js";
import {
  DEPRECATED_AGENT,
  DEPRECATED_OUTPUT,
  DEPRECATED_SKILL,
  DEPRECATED_SKILLS,
  DEPRECATED_SPEC_SKILLS,
  DEPRECATED_TARGET,
  deprecationWarnings,
  noteDeprecatedKeyword,
  packageSpecDeprecationWarnings,
} from "./deprecation.js";
import { parseManifest } from "./index.js";

describe("legacy Prayfile keyword deprecation", () => {
  it("records target, output, and agent", () => {
    const manifest = parseManifest(`
prayfile "1"
target :tool_a do
  output "INSTRUCTIONS.md"
end
agent "sample/base", "~> 1.0"
`);
    assert.deepEqual(manifest.deprecatedKeywords, [
      DEPRECATED_TARGET,
      DEPRECATED_OUTPUT,
      DEPRECATED_AGENT,
    ]);
    const warnings = deprecationWarnings(manifest.deprecatedKeywords);
    assert.equal(warnings.length, 3);
    assert.ok(warnings.every((warning) => warning.includes("version 2")));
  });

  it("records skills keyword", () => {
    const manifest = parseManifest(`
prayfile "1"
target :tool_a do
  skills ".agents/vendor"
end
tree ".agents/skills" do
end
`);
    assert.ok(manifest.deprecatedKeywords?.includes(DEPRECATED_SKILLS));
    const warnings = deprecationWarnings(manifest.deprecatedKeywords);
    assert.ok(warnings.some((warning) => warning.includes("`skills`")));
    assert.ok(warnings.some((warning) => warning.includes("`tree`")));
  });

  it("does not mark a tree dest whose path contains skills", () => {
    const manifest = parseManifest(`
prayfile "1"
tree ".agents/skills" do
  pray "sample/base", "~> 1.0"
end
`);
    assert.deepEqual(manifest.deprecatedKeywords ?? [], []);
  });

  it("warns for spec.skills and skill export type", () => {
    const spec = parsePackageSpec(`
Package::Specification.new do |spec|
  spec.name = "sample/legacy"
  spec.version = "1.0.0"
  spec.files = ["folders/review/README.md"]
  spec.exports = {
    "review" => { type: "skill", path: "folders/review" }
  }
  spec.skills = {
    "other" => { path: "folders/other", summary: "other" }
  }
end
`);
    const warnings = packageSpecDeprecationWarnings(spec);
    assert.ok(warnings.some((warning) => warning.includes("`spec.skills`")));
    assert.ok(warnings.some((warning) => warning.includes("`skill`")));
    assert.ok(warnings.every((warning) => warning.includes("version 2")));
    assert.equal(DEPRECATED_SPEC_SKILLS, "spec.skills");
    assert.equal(DEPRECATED_SKILL, "skill");
  });

  it("does not mark recommended forms", () => {
    const manifest = parseManifest(`
prayfile "1"
compose "AGENTS.md" do
  pray "sample/base", "~> 1.0"
end
`);
    assert.deepEqual(manifest.deprecatedKeywords ?? [], []);
  });

  it("deduplicates keywords", () => {
    let keywords: string[] = [];
    keywords = noteDeprecatedKeyword(keywords, DEPRECATED_AGENT);
    keywords = noteDeprecatedKeyword(keywords, DEPRECATED_AGENT);
    assert.deepEqual(keywords, [DEPRECATED_AGENT]);
  });
});
