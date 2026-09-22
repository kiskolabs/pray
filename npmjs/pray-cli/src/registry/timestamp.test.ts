import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { versionFromHash } from "./index.js";

const version = {
  version: "1.0.0",
  artifact: "v1/artifacts/sample/base/1.0.0/package.praypkg",
};

describe("registry publish timestamps", () => {
  it("normalizes legacy timestamps", () => {
    assert.equal(
      versionFromHash({
        ...version,
        published_at: "2020-01-01T00:00:00.999Z",
      }).publishedAt,
      1_577_836_800,
    );
    assert.equal(
      versionFromHash({ ...version, published_at: "1234567890" }).publishedAt,
      1_234_567_890,
    );
    assert.equal(
      versionFromHash({ ...version, published_at: null }).publishedAt,
      undefined,
    );
  });

  it("rejects non-canonical values outside migration formats", () => {
    for (const publishedAt of [1.5, -1, 253_402_300_800, "tomorrow"]) {
      assert.throws(() =>
        versionFromHash({ ...version, published_at: publishedAt }),
      );
    }
  });
});
