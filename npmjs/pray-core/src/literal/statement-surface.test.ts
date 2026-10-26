import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  expandStatementSurface,
  splitSymbolAssignment,
} from "./statement-surface.js";

describe("statement surface", () => {
  it("expands semicolon one-liner", () => {
    assert.deepEqual(
      expandStatementSurface(
        'pray do; support_email("a@example.com"); security_email("b@example.com"); end',
      ),
      [
        "pray do",
        'support_email "a@example.com"',
        'security_email "b@example.com"',
        "end",
      ],
    );
  });

  it("expands brace block", () => {
    assert.deepEqual(
      expandStatementSurface(
        'pray{support_email("a@example.com");security_email("b@example.com")}',
      ),
      [
        "pray do",
        'support_email "a@example.com"',
        'security_email "b@example.com"',
        "end",
      ],
    );
  });

  it("unwraps compose call parentheses", () => {
    assert.deepEqual(expandStatementSurface('compose("AGENTS.md") do'), [
      'compose "AGENTS.md" do',
    ]);
  });

  it("splits symbol call form", () => {
    assert.deepEqual(
      splitSymbolAssignment('support_email("contact@kiskolabs.com")'),
      {
        key: "support_email",
        value: '"contact@kiskolabs.com"',
      },
    );
  });

  it("leaves assignment map literals alone", () => {
    const statement = 'spec.exports = { "AGENTS.md" => "templates/agents.md" }';
    assert.deepEqual(expandStatementSurface(statement), [statement]);
  });
});
