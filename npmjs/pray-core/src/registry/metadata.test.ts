import assert from "node:assert/strict";
import { createServer } from "node:http";
import { describe, it } from "node:test";
import { fetchPackageMetadata } from "./index.js";

describe("registry package metadata", () => {
  it("reuses remote package metadata within the process", async () => {
    let hits = 0;
    const server = createServer((_request, response) => {
      hits += 1;
      response.writeHead(200, { "content-type": "application/json" });
      response.end(JSON.stringify({ name: "sample/base", versions: [] }));
    });
    await new Promise<void>((resolve) => {
      server.listen(0, "127.0.0.1", resolve);
    });
    const address = server.address();
    if (address === null || typeof address === "string") {
      throw new Error("expected a TCP listen address");
    }
    const sourceUrl = `http://127.0.0.1:${address.port}`;
    try {
      const first = await fetchPackageMetadata(sourceUrl, "sample/base");
      const second = await fetchPackageMetadata(sourceUrl, "sample/base");
      assert.equal(first.name, "sample/base");
      assert.equal(second.name, "sample/base");
      assert.equal(hits, 1);
    } finally {
      server.close();
    }
  });
});
