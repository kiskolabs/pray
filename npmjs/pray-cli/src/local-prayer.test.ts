import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  pathSourcePackageDirectory,
  validateLocalPrayerName,
} from "./local-prayer.js";

describe("local prayer name", () => {
  it("accepts a single folder name", () => {
    assert.equal(validateLocalPrayerName("project"), "project");
    assert.equal(validateLocalPrayerName("notes"), "notes");
  });

  it("strips a matching source handle from a path package directory", () => {
    assert.equal(
      pathSourcePackageDirectory("local", "local/project"),
      "project",
    );
    assert.equal(
      pathSourcePackageDirectory("amkisko", "amkisko/rules"),
      "rules",
    );
    assert.equal(
      pathSourcePackageDirectory("local", "amkisko/rules"),
      "amkisko-rules",
    );
  });

  it("refuses the distribution layout name", () => {
    assert.throws(() => validateLocalPrayerName("v1"), /reserved/);
  });
});
