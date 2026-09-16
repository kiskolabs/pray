import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { parsePackageSpec } from "./index.js";
import { renderPackageSpec } from "./render.js";

const COMPLETE_SPEC = `
Package::Specification.new do |spec|
  spec.name = "fork/base"
  spec.version = "1.0.0"
  spec.summary = "summary"
  spec.description = "description"
  spec.authors = ["Author"]
  spec.maintainers = ["Kim"]
  spec.license = "MIT"
  spec.homepage = "https://example.com"
  spec.source_code_uri = "https://example.com/source"
  spec.changelog_uri = "https://example.com/changelog"
  spec.prayfile_version = "1"
  spec.files = ["a.md"]
  spec.exports = {
    "a" => {
      type: "fragment",
      path: "a.md",
      summary: "A",
      only: ["tool_a"],
      except: ["tool_b"],
      default_path: "A.md"
    }
  }
  spec.skills = {
    "review" => { path: "skills/review", summary: "Review" }
  }
  spec.templates = {
    "note" => { path: "templates/note.md", summary: "Note" }
  }
  spec.adapters = { "tool" => "adapter" }
  spec.targets = ["tool_a"]
  spec.add_optional_dependency "sample/common", "~> 1.0"
  spec.metadata = { "labels" => ["stable", :tool-a], "policy" => { "enabled" => true, "fallback" => nil }, "priority" => 1 }
  spec.upstream "sample/base", "~> 1.4"
end
`;

describe("package spec render", () => {
  it("parses maintainers and the pray_version alias", () => {
    const spec = parsePackageSpec(`
Package::Specification.new do |spec|
  spec.name = "sample/base"
  spec.version = "1.4.3"
  spec.authors = ["Pat"]
  spec.maintainers = ["Kim", "Alex"]
  spec.pray_version = ">= 0.1"
  spec.files = ["README.md"]
end
`);
    assert.deepEqual(spec.authors, ["Pat"]);
    assert.deepEqual(spec.maintainers, ["Alex", "Kim"]);
    assert.equal(spec.prayfileVersion, ">= 0.1");
  });

  it("round-trips a fork spec with upstream", () => {
    const spec = parsePackageSpec(`
Package::Specification.new do |spec|
  spec.name = "fork/base"
  spec.version = "1.0.0"
  spec.summary = "forked guidance"
  spec.files = ["README.md", "exports/a.md"]
  spec.exports = {
    "a" => {
      type: "fragment",
      path: "exports/a.md",
      summary: "A"
    }
  }
  spec.upstream "sample/base", "~> 1.4"
end
`);
    const parsed = parsePackageSpec(renderPackageSpec(spec));
    assert.equal(parsed.name, "fork/base");
    assert.equal(parsed.upstream?.name, "sample/base");
    assert.equal(parsed.upstream?.constraint, "~> 1.4");
    assert.equal(parsed.exports.get("a")?.path, "exports/a.md");
  });

  it("round-trips every supported field", () => {
    const expected = parsePackageSpec(COMPLETE_SPEC);
    const parsed = parsePackageSpec(renderPackageSpec(expected));
    assert.equal(parsed.homepage, "https://example.com");
    assert.equal(parsed.sourceCodeUri, "https://example.com/source");
    assert.equal(parsed.changelogUri, "https://example.com/changelog");
    assert.equal(parsed.prayfileVersion, "1");
    assert.deepEqual(parsed.maintainers, ["Kim"]);
    assert.equal(parsed.exports.get("a")?.defaultPath, "A.md");
    assert.equal(parsed.skills.get("review")?.path, "skills/review");
    assert.equal(parsed.templates.get("note")?.path, "templates/note.md");
    assert.equal(parsed.adapters.get("tool"), "adapter");
    assert.equal(parsed.dependencies[0]?.optional, true);
    assert.equal(parsed.metadata.get("priority")?.kind, "integer");
    assert.deepEqual(parsed, expected);
  });

  it("omits an empty version", () => {
    const spec = parsePackageSpec(`
Package::Specification.new do |spec|
  spec.name = "project"
  spec.files = ["exports/project.md"]
end
`);
    const rendered = renderPackageSpec(spec);
    assert.equal(rendered.includes("spec.version"), false);
  });
});
