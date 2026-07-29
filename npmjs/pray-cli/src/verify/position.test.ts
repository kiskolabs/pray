import assert from "node:assert/strict";
import { describe, it } from "node:test";
import type { ManagedSpanRecord } from "../lockfile/types.js";
import type { ResolvedLocalFile } from "../resolve/types.js";
import {
  formatPositionDriftMessage,
  summarizePositionDrift,
} from "./position.js";

function span(
  id: string,
  open: number,
  close: number,
  checksum: string,
): ManagedSpanRecord {
  return {
    id,
    target: "AGENTS.md",
    open_line: open,
    close_line: close,
    ideal_checksum: checksum,
    package: "sample/base",
    export: "guidance",
    source_checksum: "sha256:source",
    silenced: false,
  };
}

describe("summarizePositionDrift", () => {
  it("groups uniform position drift with local cause", () => {
    const spans = [
      span("aaaa1111", 4, 6, "sha256:one"),
      span("bbbb2222", 8, 10, "sha256:two"),
    ];
    const markers = new Map([
      ["aaaa1111", { openLine: 6, closeLine: 8, checksum: "sha256:one" }],
      ["bbbb2222", { openLine: 10, closeLine: 12, checksum: "sha256:two" }],
    ]);
    const onDisk = [
      "# Title",
      "",
      "Local alpha",
      "Extra unmarked",
      "",
      "Local beta",
      "<!-- pray:aaaa1111 -->",
      "body one",
      "<!-- pray:aaaa1111 -->",
      "",
      "<!-- pray:bbbb2222 -->",
      "body two",
      "<!-- pray:bbbb2222 -->",
    ];
    const fresh = [
      "# Title",
      "",
      "Local alpha",
      "Local beta",
      "<!-- pray:aaaa1111 -->",
      "body one",
      "<!-- pray:aaaa1111 -->",
      "",
      "<!-- pray:bbbb2222 -->",
      "body two",
      "<!-- pray:bbbb2222 -->",
    ];
    const locals: ResolvedLocalFile[] = [
      {
        path: ".agents/project.md",
        manifestPath: ".agents/project.md",
        content: "Local alpha\nLocal beta\n",
        position: "before",
        optional: false,
      },
    ];
    const summary = summarizePositionDrift(
      "AGENTS.md",
      spans,
      markers,
      onDisk,
      fresh,
      locals,
    );
    assert.ok(summary);
    assert.equal(summary.markerCount, 2);
    assert.equal(summary.uniformDelta, 2);
    assert.equal(summary.firstId, "aaaa1111");
    const message = formatPositionDriftMessage(summary);
    assert.match(
      message,
      /`AGENTS\.md` position drift \(\+2 lines\) across 2 markers/,
    );
    assert.match(message, /first marker `aaaa1111` lock 4:6, file 6:8/);
    assert.match(
      message,
      /cause: `AGENTS\.md:4` unmarked text differs from `\.agents\/project\.md:2`/,
    );
  });

  it("skips checksum mismatched spans", () => {
    const spans = [span("aaaa1111", 2, 4, "sha256:ideal")];
    const markers = new Map([
      ["aaaa1111", { openLine: 3, closeLine: 5, checksum: "sha256:edited" }],
    ]);
    const onDisk = [
      "text",
      "<!-- pray:aaaa1111 -->",
      "edited",
      "<!-- pray:aaaa1111 -->",
    ];
    assert.equal(
      summarizePositionDrift(
        "AGENTS.md",
        spans,
        markers,
        onDisk,
        undefined,
        [],
      ),
      undefined,
    );
  });
});
