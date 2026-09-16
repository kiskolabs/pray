import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { PrayError } from "../errors.js";
import { ensureLockedUpstreamMatches } from "./upstream.js";
import {
  mergeContentFiles,
  nextUpstreamConstraint,
  overlayDriftLine,
  overlayFileChanges,
  tryMergeContentFiles,
  upstreamMergeConflictMessage,
} from "./upstream-merge.js";

describe("package upstream", () => {
  it("rejects content that no longer matches the locked upstream", () => {
    assert.throws(
      () =>
        ensureLockedUpstreamMatches(
          {
            name: "sample/base",
            version: "1.4.3",
            source: "sample",
            tree_hash: "sha256:old",
            artifact_hash: "sha256:artifact",
          },
          {
            name: "sample/base",
            version: "1.4.3",
            source: "sample",
            tree_hash: "sha256:changed",
            artifact_hash: "sha256:artifact",
          },
        ),
      /locked upstream tree hash mismatch/,
    );
  });

  it("takes the new tree for a clean replica", () => {
    const oldContent = new Map([["exports/a.md", Buffer.from("old")]]);
    const newContent = new Map([["exports/a.md", Buffer.from("new")]]);
    const merged = mergeContentFiles(oldContent, newContent, oldContent);
    assert.equal(merged.get("exports/a.md")?.toString(), "new");
  });

  it("conflicts when a local edit overlaps an upstream change", () => {
    const oldContent = new Map([["exports/a.md", Buffer.from("old")]]);
    const newContent = new Map([["exports/a.md", Buffer.from("new")]]);
    const localContent = new Map([["exports/a.md", Buffer.from("edit")]]);
    assert.throws(
      () => mergeContentFiles(oldContent, newContent, localContent),
      (error: unknown) =>
        error instanceof PrayError && error.message.includes("exports/a.md"),
    );
  });

  it("names the fork and every conflicting path", () => {
    const oldContent = new Map([
      ["README.md", Buffer.from("old readme")],
      ["exports/a.md", Buffer.from("old")],
    ]);
    const newContent = new Map([
      ["README.md", Buffer.from("new readme")],
      ["exports/a.md", Buffer.from("new")],
    ]);
    const localContent = new Map([
      ["README.md", Buffer.from("local readme")],
      ["exports/a.md", Buffer.from("edit")],
    ]);
    const paths = tryMergeContentFiles(oldContent, newContent, localContent);
    assert.ok(Array.isArray(paths));
    assert.ok(paths.includes("README.md"));
    assert.ok(paths.includes("exports/a.md"));
    const message = upstreamMergeConflictMessage(
      "fork/base",
      "sample/base",
      "1.4.3",
      "1.4.4",
      paths,
    );
    assert.match(message, /fork\/base/);
    assert.match(message, /sample\/base 1.4.3 to 1.4.4/);
    assert.match(message, /README.md/);
    assert.match(message, /exports\/a.md/);
  });

  it("keeps a local edit when upstream is unchanged", () => {
    const oldContent = new Map([["exports/a.md", Buffer.from("old")]]);
    const localContent = new Map([["exports/a.md", Buffer.from("edit")]]);
    const merged = mergeContentFiles(oldContent, oldContent, localContent);
    assert.equal(merged.get("exports/a.md")?.toString(), "edit");
  });

  it("rewrites an exact upstream pin to the new version", () => {
    assert.equal(nextUpstreamConstraint("= 1.4.3", "1.4.4"), "= 1.4.4");
    assert.equal(nextUpstreamConstraint("1.4.3", "1.4.4"), "= 1.4.4");
    assert.equal(nextUpstreamConstraint("~> 1.4", "1.4.4"), "~> 1.4");
  });

  it("omits a file removed from upstream when local still matches old", () => {
    const oldContent = new Map([["gone.md", Buffer.from("old")]]);
    const newContent = new Map();
    const localContent = new Map([["gone.md", Buffer.from("old")]]);
    const merged = mergeContentFiles(oldContent, newContent, localContent);
    assert.equal(merged.has("gone.md"), false);
  });

  it("names overlay drift for changed local-only and missing paths", () => {
    const localContent = new Map([
      ["README.md", Buffer.from("edit")],
      ["extra.md", Buffer.from("local")],
    ]);
    const upstreamContent = new Map([
      ["README.md", Buffer.from("base")],
      ["gone.md", Buffer.from("upstream")],
    ]);
    const changes = overlayFileChanges(localContent, upstreamContent);
    assert.deepEqual(changes, [
      ["README.md", "changed"],
      ["extra.md", "local_only"],
      ["gone.md", "missing"],
    ]);
    assert.match(
      overlayDriftLine(
        "fork/base",
        "sample/base",
        "1.4.3",
        "README.md",
        "changed",
      ),
      /differs/,
    );
  });
});
