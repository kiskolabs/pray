import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { PrayError } from "./errors.js";
import { substitutePraySymbols } from "./substitute.js";

describe("substitutePraySymbols", () => {
  it("replaces known symbols", () => {
    assert.equal(
      substitutePraySymbols("Email ((pray:support_email))", {
        support_email: "contact@example.com",
      }),
      "Email contact@example.com",
    );
  });

  it("ignores spaced forms", () => {
    assert.equal(
      substitutePraySymbols("(( pray:support_email ))", {
        support_email: "contact@example.com",
      }),
      "(( pray:support_email ))",
    );
  });

  it("rejects unknown symbols", () => {
    assert.throws(
      () => substitutePraySymbols("((pray:missing))", {}),
      (error: unknown) =>
        error instanceof PrayError &&
        error.kind === "render" &&
        error.message.includes("unknown pray symbol"),
    );
  });
});
