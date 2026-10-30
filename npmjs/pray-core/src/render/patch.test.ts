import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { checksumManagedSpanContent } from "../hashing.js";
import { patchRenderedContent } from "./patch.js";

describe("patchRenderedContent", () => {
  it("appends a trailing managed span with the blank line from fresh", () => {
    const existing = `\
<!-- pray:abc123 -->
package body
<!-- pray:abc123 -->
`;
    const fresh = `\
<!-- pray:abc123 -->
package body
<!-- pray:abc123 -->

<!-- pray:local001 -->
new local body
<!-- pray:local001 -->
`;
    assert.equal(patchRenderedContent(existing, fresh), fresh);
  });

  it("restores a missing blank line between adjacent managed spans", () => {
    const existing = `\
<!-- pray:abc123 -->
package body
<!-- pray:abc123 -->
<!-- pray:local001 -->
new local body
<!-- pray:local001 -->
`;
    const fresh = `\
<!-- pray:abc123 -->
package body
<!-- pray:abc123 -->

<!-- pray:local001 -->
new local body
<!-- pray:local001 -->
`;
    assert.equal(patchRenderedContent(existing, fresh), fresh);
  });

  it("drops unchanged retired and preserves edited and unknown spans", () => {
    const existing = `\
<!-- pray:retired1 -->
retired body
<!-- pray:retired1 -->
<!-- pray:edited01 -->
edited body
<!-- pray:edited01 -->
<!-- pray:unknown1 -->
unknown body
<!-- pray:unknown1 -->
`;
    const retired = new Map([
      ["retired1", checksumManagedSpanContent("retired body\n")],
      ["edited01", checksumManagedSpanContent("original body\n")],
    ]);

    const patched = patchRenderedContent(existing, "", retired);

    assert.doesNotMatch(patched, /retired body/);
    assert.match(patched, /edited body/);
    assert.match(patched, /unknown body/);
  });
});
