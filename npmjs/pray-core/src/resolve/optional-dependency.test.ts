import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, it } from "node:test";
import { resolveProject } from "./project.js";

function writePackage(
  root: string,
  leaf: string,
  version: string,
  optional?: { name: string; constraint: string },
): void {
  const packageRoot = join(root, "packages", leaf);
  mkdirSync(packageRoot, { recursive: true });
  writeFileSync(join(packageRoot, "README.md"), `${leaf}\n`);
  const dependency = optional
    ? `  spec.add_optional_dependency "${optional.name}", "${optional.constraint}"\n`
    : "";
  writeFileSync(
    join(packageRoot, `${leaf}.prayspec`),
    `Package::Specification.new do |spec|\n  spec.name = "sample/${leaf}"\n  spec.version = "${version}"\n  spec.files = ["README.md"]\n${dependency}end\n`,
  );
}

function writeManifest(root: string, leaves: string[]): string {
  const declarations = leaves
    .map((leaf) => `pray "sample/${leaf}", path: "packages/${leaf}"`)
    .join("\n");
  const path = join(root, "Prayfile");
  writeFileSync(path, `prayfile "1"\n${declarations}\n`);
  return path;
}

describe("optional dependency referrals", () => {
  it("accepts mutual referrals", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-mutual-optional-"));
    try {
      writePackage(root, "alpha", "1.0.0", {
        name: "sample/beta",
        constraint: "~> 1.0",
      });
      writePackage(root, "beta", "1.0.0", {
        name: "sample/alpha",
        constraint: "~> 1.0",
      });

      const project = await resolveProject(
        writeManifest(root, ["alpha", "beta"]),
      );
      assert.equal(project.packages.length, 2);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it("checks a referral constraint when the target is selected", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-optional-constraint-"));
    try {
      writePackage(root, "alpha", "1.0.0", {
        name: "sample/beta",
        constraint: "~> 2.0",
      });
      writePackage(root, "beta", "1.0.0");

      await assert.rejects(
        resolveProject(writeManifest(root, ["alpha", "beta"])),
        /optional referral sample\/alpha -> sample\/beta requires ~> 2\.0, but selected version is 1\.0\.0/,
      );
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it("does not select an absent target", async () => {
    const root = mkdtempSync(join(tmpdir(), "pray-optional-absent-"));
    try {
      writePackage(root, "alpha", "1.0.0", {
        name: "sample/missing",
        constraint: "~> 1.0",
      });

      const project = await resolveProject(writeManifest(root, ["alpha"]));
      assert.equal(project.packages.length, 1);
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });
});
