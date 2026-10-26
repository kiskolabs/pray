import assert from "node:assert/strict";
import {
  mkdirSync,
  mkdtempSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { it } from "node:test";
import { resolveProject } from "./resolve/project.js";

it("rejects a package directory reached through a symbolic link", async () => {
  const root = mkdtempSync(join(tmpdir(), "pray-package-root-"));
  const outside = mkdtempSync(join(tmpdir(), "pray-package-root-outside-"));
  try {
    mkdirSync(join(outside, "base/exports"), { recursive: true });
    writeFileSync(join(outside, "base/exports/a.md"), "outside\n");
    writeFileSync(
      join(outside, "base/base.prayspec"),
      'Package::Specification.new do |spec|\n spec.name = "sample/package"\n spec.version = "1.0.0"\n spec.files = ["exports/a.md"]\n spec.exports = { "a" => { type: "fragment", path: "exports/a.md" } }\nend\n',
    );
    symlinkSync(join(outside, "base"), join(root, "package"));
    writeFileSync(
      join(root, "Prayfile"),
      'prayfile "1"\ncompose "AGENTS.md" do\n pray "sample/package", path: "package"\nend\n',
    );

    await assert.rejects(
      resolveProject(join(root, "Prayfile")),
      /package directory uses a symbolic link/,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
    rmSync(outside, { recursive: true, force: true });
  }
});

it("rejects package files reached through symbolic links", async () => {
  const root = mkdtempSync(join(tmpdir(), "pray-source-symlink-"));
  try {
    mkdirSync(join(root, "package/exports"), { recursive: true });
    writeFileSync(join(root, "outside.md"), "outside\n");
    symlinkSync(join(root, "outside.md"), join(root, "package/exports/a.md"));
    writeFileSync(
      join(root, "package/package.prayspec"),
      'Package::Specification.new do |spec|\n spec.name = "sample/package"\n spec.version = "1.0.0"\n spec.files = ["exports/a.md"]\n spec.exports = { "a" => { type: "fragment", path: "exports/a.md" } }\nend\n',
    );
    writeFileSync(
      join(root, "Prayfile"),
      'prayfile "1"\ncompose "AGENTS.md" do\n pray "sample/package", path: "package"\nend\n',
    );

    await assert.rejects(
      resolveProject(join(root, "Prayfile")),
      /symbolic link/,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

it("rejects local compose files reached through symbolic links", async () => {
  const root = mkdtempSync(join(tmpdir(), "pray-local-symlink-"));
  try {
    writeFileSync(join(root, "outside.md"), "outside\n");
    symlinkSync(join(root, "outside.md"), join(root, "local.md"));
    writeFileSync(
      join(root, "Prayfile"),
      'prayfile "1"\ncompose "AGENTS.md" do\n local "local.md"\nend\n',
    );

    await assert.rejects(
      resolveProject(join(root, "Prayfile")),
      /symbolic link/,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
