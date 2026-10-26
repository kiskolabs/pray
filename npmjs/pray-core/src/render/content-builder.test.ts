import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { ContentBuilder } from "./content-builder.js";

describe("ContentBuilder", () => {
  it("tracks the next line while appending lines and bodies", () => {
    const builder = new ContentBuilder();

    assert.equal(builder.nextLineNumber(), 1);
    builder.appendLine("one");
    builder.appendBody("two\nthree\n");
    assert.equal(builder.nextLineNumber(), 4);
  });

  it("counts empty lines and keeps one trailing newline", () => {
    const builder = new ContentBuilder();
    builder.appendLine("one");
    builder.appendEmptyLine();

    assert.equal(builder.nextLineNumber(), 3);
    assert.equal(builder.finish(), "one\n");
  });
});
