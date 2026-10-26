import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { describe, it } from "node:test";
import { fileURLToPath } from "node:url";
import { PrayError } from "./errors.js";
import { parseLockfile } from "./lockfile/index.js";
import { parseManifest } from "./manifest/index.js";
import { parsePackageSpec } from "./package-spec/index.js";
import { renderProject } from "./render/project.js";
import { defaultResolveOptions, resolveProject } from "./resolve/project.js";
import { inspectLockedDestinations } from "./verify/locked-dest.js";

const here = dirname(fileURLToPath(import.meta.url));
const fixturesRoot = join(here, "../../../fixtures");

describe("RFC 0100 conformance fixtures", () => {
  it("parses the minimal Prayfile fixture", () => {
    const dir = join(fixturesRoot, "parser/minimal-prayfile");
    const expected = JSON.parse(
      readFileSync(join(dir, "expected.json"), "utf8"),
    );
    const manifest = parseManifest(readFileSync(join(dir, "Prayfile"), "utf8"));
    assert.equal(manifest.prayfileVersion, expected.prayfile_version);
    assert.deepEqual(
      manifest.packages.map((entry) => entry.name),
      expected.package_names,
    );
    assert.deepEqual(
      manifest.targets.map((entry) => entry.name),
      expected.target_names,
    );
  });

  it("parses the minimal prayspec fixture", () => {
    const dir = join(fixturesRoot, "prayspec/minimal-package");
    const expected = JSON.parse(
      readFileSync(join(dir, "expected.json"), "utf8"),
    );
    const spec = parsePackageSpec(
      readFileSync(join(dir, "sample-base.prayspec"), "utf8"),
    );
    assert.equal(spec.name, expected.name);
    assert.equal(spec.version, expected.version);
    assert.deepEqual(spec.files, expected.files);
  });

  it("parses the minimal lockfile fixture", () => {
    const dir = join(fixturesRoot, "lockfile/minimal");
    const expected = JSON.parse(
      readFileSync(join(dir, "expected.json"), "utf8"),
    );
    const lockfile = parseLockfile(
      readFileSync(join(dir, "Prayfile.lock"), "utf8"),
    );
    assert.equal(lockfile.prayfile_lock, expected.prayfile_lock);
    assert.equal(lockfile.spec, expected.spec);
    assert.deepEqual(
      lockfile.package.map((entry) => entry.name),
      expected.package_names,
    );
    assert.deepEqual(
      lockfile.managed_span.map((entry) => entry.id),
      expected.managed_span_ids,
    );
  });

  it("rejects the invalid lockfile fixture", () => {
    const text = readFileSync(
      join(fixturesRoot, "lockfile/invalid-toml/Prayfile.lock"),
      "utf8",
    );
    assert.throws(() => parseLockfile(text), PrayError);
  });

  it("renders the compose fragment fixture to the expected dest", async () => {
    const dir = join(fixturesRoot, "render/compose-fragment");
    const expected = JSON.parse(
      readFileSync(join(dir, "expected.json"), "utf8"),
    );
    const project = await resolveProject(join(dir, "Prayfile"), {
      ...defaultResolveOptions(),
      offline: true,
    });
    const rendered = renderProject(project);
    assert.deepEqual(
      project.packages.map((entry) => entry.declaration.name),
      expected.package_names,
    );
    assert.equal(rendered.length, 1);
    assert.equal(rendered[0]?.path, expected.dest_path);
    const dest = readFileSync(
      join(dir, "expected", expected.dest_path),
      "utf8",
    );
    assert.equal(rendered[0]?.content, dest);
  });

  for (const pack of [
    "span-matching",
    "span-edited",
    "span-missing-dest",
    "span-removed",
    "span-orphan",
  ]) {
    it(`inspects locked destinations for ${pack}`, () => {
      const dir = join(fixturesRoot, `lockfile/${pack}`);
      const expected = JSON.parse(
        readFileSync(join(dir, "expected.json"), "utf8"),
      );
      const lockfile = parseLockfile(
        readFileSync(join(dir, "Prayfile.lock"), "utf8"),
      );
      const report = inspectLockedDestinations(dir, lockfile);
      const kinds = report.findings.map((finding) => finding.kind).sort();
      assert.deepEqual(kinds, [...expected.finding_kinds].sort());
    });
  }
});
