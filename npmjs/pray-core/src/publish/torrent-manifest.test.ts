import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { sha256Prefixed } from "../hashing.js";
import { torrentManifestPayload } from "./torrent-manifest.js";

describe("torrentManifestPayload", () => {
  it("hashes pieces at 16KiB with a sha256 prefix", () => {
    const bytes = Buffer.alloc(16 * 1024 + 1, 0x61);
    const payload = torrentManifestPayload(
      "sample/base",
      "1.4.3",
      "v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg",
      bytes,
      ["http://tracker.example/announce"],
    );

    assert.equal(payload.spec, "pray-torrent-v1");
    assert.equal(payload.piece_size, 16_384);
    assert.equal(payload.length, bytes.length);
    assert.equal(payload.pieces.length, 2);
    assert.equal(payload.pieces[0], sha256Prefixed(Buffer.alloc(16_384, 0x61)));
    assert.equal(payload.pieces[1], sha256Prefixed(Buffer.from("a")));
    assert.equal(payload.artifact_hash, sha256Prefixed(bytes));
    assert.deepEqual(payload.sources, [
      "v1/artifacts/sample/base/1.4.3/sample-base-1.4.3.praypkg",
    ]);
    assert.deepEqual(payload.trackers, ["http://tracker.example/announce"]);
  });

  it("emits no pieces for an empty artifact", () => {
    const payload = torrentManifestPayload(
      "sample/base",
      "1.0.0",
      "v1/artifacts/empty.praypkg",
      Buffer.alloc(0),
      [],
    );
    assert.equal(payload.length, 0);
    assert.deepEqual(payload.pieces, []);
  });
});
