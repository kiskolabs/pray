import assert from "node:assert/strict";
import { describe, it } from "node:test";
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
});
