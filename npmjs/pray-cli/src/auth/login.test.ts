import assert from "node:assert/strict";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { PrayError } from "../errors.js";
import { parseLoginArguments } from "./login.js";
import { loadSessions, persistSession, sessionFilePath } from "./session.js";

describe("login parsing", () => {
  it("requires server email and exactly one auth mode", () => {
    assert.throws(
      () => parseLoginArguments(["--email", "a@example.com"]),
      (error: unknown) =>
        error instanceof PrayError &&
        error.message.includes("at least one --server"),
    );
    assert.throws(
      () =>
        parseLoginArguments([
          "--server",
          "http://127.0.0.1:9",
          "--email",
          "a@example.com",
        ]),
      (error: unknown) =>
        error instanceof PrayError &&
        error.message.includes("exactly one authentication mode"),
    );
  });

  it("accepts passkey mode", () => {
    const options = parseLoginArguments([
      "--server",
      "http://127.0.0.1:9",
      "--email",
      "a@example.com",
      "--passkey-key",
      "/tmp/key",
      "--credential-id",
      "cred-1",
    ]);
    assert.equal(options.passkeyKey, "/tmp/key");
    assert.equal(options.credentialId, "cred-1");
    assert.equal(options.sshAgent, false);
  });
});

describe("session persistence", () => {
  it("stores and replaces sessions by server url", () => {
    const root = mkdtempSync(join(tmpdir(), "pray-session-"));
    try {
      persistSession(root, {
        server_url: "http://a",
        email: "a@example.com",
        token: "t1",
        kind: "passkey",
      });
      persistSession(root, {
        server_url: "http://b",
        email: "b@example.com",
        token: "t2",
        kind: "ssh_key",
      });
      persistSession(root, {
        server_url: "http://a",
        email: "a2@example.com",
        token: "t3",
        kind: "passkey",
      });
      const sessions = loadSessions(sessionFilePath(root));
      assert.equal(sessions?.length, 2);
      assert.equal(
        sessions?.find((entry) => entry.server_url === "http://a")?.email,
        "a2@example.com",
      );
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});
