import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  DEPRECATED_AGENT,
  DEPRECATED_OUTPUT,
  DEPRECATED_TARGET,
  deprecationWarnings,
  noteDeprecatedKeyword,
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
