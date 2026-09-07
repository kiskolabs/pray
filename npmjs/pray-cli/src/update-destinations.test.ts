import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, it } from "node:test";
import { fileURLToPath } from "node:url";
import { readLockfile, writeLockfile } from "./lockfile/index.js";

class UpdateFixture {
  readonly root = mkdtempSync(join(tmpdir(), "pray-update-"));
  readonly consumer = join(this.root, "consumer");
  readonly source = join(this.root, "source");
  readonly distribution = join(this.root, "distribution");
  readonly environment = {
    ...process.env,
    PRAY_PATH: undefined,
    PRAY_FILE_PATH: undefined,
    PRAY_ENV: undefined,
    PRAY_HOME: join(this.root, "home"),
    PRAY_CACHE: join(this.root, "cache"),
    PRAY_TRUST_ASSUME_YES: "1",
  };

  constructor(constraint: string) {
    for (const directory of [
      "source/package/exports",
      "consumer/rules",
      "distribution",
    ])
      mkdirSync(join(this.root, directory), { recursive: true });
    writeFileSync(
      join(this.source, "Prayfile"),
      'prayfile "1"\npray "sample/files", ">= 1.0", path: "package"\n',
    );
    this.git(["init", "-b", "main"]);
    this.git(["config", "user.name", "Pray Test"]);
    this.git(["config", "user.email", "pray@example.invalid"]);
    this.publish("1.0.0");
    writeFileSync(
      join(this.consumer, "rules/rules.prayspec"),
      `Package::Specification.new do |spec|
  spec.name = "sample/rules"
  spec.version = "1.0.0"
  spec.files = ["rules.md"]
  spec.exports = { "rules" => { type: "fragment", path: "rules.md" } }
end
`,
    );
    writeFileSync(join(this.consumer, "rules/rules.md"), "old rules\n");
    writeFileSync(
      join(this.consumer, "Prayfile"),
      `prayfile "1"
source "dist", "git+file://${this.distribution}"
tree "skills" do
  pray "sample/files", "${constraint}", source: "dist"
end
compose "INSTRUCTIONS.md" do
  pray "sample/rules", "~> 1.0", path: "rules"
end
`,
    );
    this.success(["install"]);
  }

  publish(version: string): void {
    writeFileSync(
      join(this.source, "package/files.prayspec"),
      `Package::Specification.new do |spec|
  spec.name = "sample/files"
  spec.version = "${version}"
  spec.files = ["exports/a.md", "exports/b.md"]
  spec.exports = { "files" => { type: "folder", path: "exports" } }
end
`,
    );
    for (const name of ["a.md", "b.md"])
      writeFileSync(join(this.source, "package/exports", name), version);
    const output = this.run(
      ["publish", "--root", join(this.distribution, "prayers")],
      this.source,
    );
    assert.equal(output.status, 0, output.stderr);
    this.git(["add", "-A"]);
    this.git(["commit", "-m", version]);
  }

  omitLedger(): void {
    const path = join(this.consumer, "Prayfile.lock");
    const lockfile = readLockfile(path);
    lockfile.provisioned = [];
    lockfile.generated_by = "pray 1.9.1";
    writeLockfile(path, lockfile);
  }

  run(argumentsList: string[], directory = this.consumer) {
    return spawnSync(
      process.execPath,
      [
        fileURLToPath(new URL("../bin/pray.js", import.meta.url)),
        ...argumentsList,
      ],
      {
        cwd: directory,
        env: this.environment,
        encoding: "utf8",
      },
    );
  }

  success(argumentsList: string[]) {
    const output = this.run(argumentsList);
    assert.equal(output.status, 0, output.stderr);
    return output;
  }

  private git(argumentsList: string[]): void {
    const output = spawnSync(
      "git",
      [
        "-c",
        "commit.gpgsign=false",
        "-c",
        "core.hooksPath=/dev/null",
        ...argumentsList,
      ],
      {
        cwd: this.distribution,
        encoding: "utf8",
      },
    );
    assert.equal(output.status, 0, output.stderr);
  }
}

let fixture: UpdateFixture;
afterEach(() => {
  if (fixture) rmSync(fixture.root, { recursive: true, force: true });
});

it("preserves the recipe and compose files and reports every collision", () => {
  fixture = new UpdateFixture("~> 1.0");
  fixture.omitLedger();
  const files = ["Prayfile", "Prayfile.lock", "INSTRUCTIONS.md"];
  const before = files.map((path) =>
    readFileSync(join(fixture.consumer, path)),
  );
  fixture.publish("2.0.0");
  writeFileSync(join(fixture.consumer, "rules/rules.md"), "new rules\n");
  for (const flags of [
    ["update", "--latest"],
    ["update", "--latest", "--json"],
  ]) {
    const output = fixture.run(flags);
    assert.equal(output.status, 5, output.stderr);
    files.forEach((path, index) => {
      assert.deepEqual(
        readFileSync(join(fixture.consumer, path)),
        before[index],
      );
    });
    for (const expected of ["a.md", "b.md", "sample/files", "move"])
      assert.ok(output.stderr.includes(expected), output.stderr);
  }
  fixture.success(["install"]);
  fixture.success(["update", "--latest"]);
  assert.equal(
    readLockfile(join(fixture.consumer, "Prayfile.lock")).package.find(
      (entry) => entry.name === "sample/files",
    )?.version,
    "2.0.0",
  );
});

it("checks candidate destinations in latest dry-run without writes", () => {
  fixture = new UpdateFixture("~> 1.0");
  fixture.omitLedger();
  const before = readFileSync(join(fixture.consumer, "Prayfile"));
  fixture.publish("2.0.0");
  const output = fixture.run(["update", "--latest", "--dry-run"]);
  assert.equal(output.status, 5, output.stderr);
  assert.ok(
    output.stderr.includes("a.md") && output.stderr.includes("b.md"),
    output.stderr,
  );
  assert.deepEqual(readFileSync(join(fixture.consumer, "Prayfile")), before);
});

it("updates in JSON mode when the constraint already admits the new version", () => {
  fixture = new UpdateFixture(">= 1.0");
  fixture.publish("2.0.0");
  const output = fixture.success(["update", "--latest", "--json"]);
  assert.equal(JSON.parse(output.stdout).status, "updated");
  assert.equal(
    readLockfile(join(fixture.consumer, "Prayfile.lock")).package.find(
      (entry) => entry.name === "sample/files",
    )?.version,
    "2.0.0",
  );
});

it("gives a recovery that preserves an edited destination", () => {
  fixture = new UpdateFixture("~> 1.0");
  const lockfile = readLockfile(join(fixture.consumer, "Prayfile.lock"));
  const destination = join(fixture.consumer, lockfile.provisioned![0]!.path);
  writeFileSync(destination, "operator changes");
  const output = fixture.run(["verify"]);
  assert.equal(output.status, 6, output.stderr);
  assert.ok(output.stderr.includes("move"), output.stderr);
  renameSync(destination, `${destination}.saved`);
  fixture.success(["install"]);
  fixture.success(["verify"]);
  assert.equal(
    readFileSync(`${destination}.saved`, "utf8"),
    "operator changes",
  );
});
