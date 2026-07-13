import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { PrayError } from "./errors.js";
import { parseTrustPolicyValue } from "./trust/parse.js";

describe("trust policy", () => {
  it("parses default and rules", () => {
    const policy = parseTrustPolicyValue({
      default: { allow: true, require_signed_commit: false },
      rules: [{ match_prefix: "https://example.com", allow: false }],
    });
    assert.equal(policy.default.allow, true);
    assert.equal(policy.rules[0]?.match_prefix, "https://example.com");
  });

  it("rejects invalid rules shape", () => {
    assert.throws(
      () => parseTrustPolicyValue({ rules: "bad" }),
      (error: unknown) =>
        error instanceof PrayError &&
        error.kind === "parse" &&
        error.message.includes("trust policy"),
    );
  });
});
