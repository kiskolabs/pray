import assert from "node:assert/strict";
import test from "node:test";
import { PrayError } from "../errors.js";
import {
  annotateGitMaterializeFailure,
  requireRevisionPresent,
} from "./materialize-check.js";

test("missing revision names git_revision_unfetched", () => {
  assert.throws(
    () =>
      requireRevisionPresent(
        "/tmp/pray-missing-git-db",
        "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
      ),
    (error: unknown) => {
      assert.ok(error instanceof PrayError);
      assert.match(error.message, /git_revision_unfetched/);
      assert.match(error.message, /pray install --locked/);
      return true;
    },
  );
});

test("missing revision offline names offline mode", () => {
  assert.throws(
    () =>
      requireRevisionPresent(
        "/tmp/pray-missing-git-db",
        "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
        true,
      ),
    (error: unknown) => {
      assert.ok(error instanceof PrayError);
      assert.match(error.message, /git_revision_unfetched/);
      assert.match(error.message, /offline mode is enabled/);
      return true;
    },
  );
});

test("materialize failure names git_cache_stale", () => {
  assert.throws(
    () =>
      annotateGitMaterializeFailure(
        "/tmp/pray-stale-git-db",
        "abc123",
        new Error("git archive failed"),
      ),
    (error: unknown) => {
      assert.ok(error instanceof PrayError);
      assert.match(error.message, /git_cache_stale/);
      assert.match(error.message, /pray install --locked/);
      return true;
    },
  );
});
