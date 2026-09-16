import assert from "node:assert/strict";
import { describe, it } from "node:test";
import type { Lockfile } from "../lockfile/types.js";
import { LOCAL_EMBED_PACKAGE } from "../lockfile/types.js";
import type { ResolvedLocalFile, ResolvedProject } from "../resolve/types.js";
import { localSummaryLines, outdatedLocalLines } from "./apply-report.js";

function localFile(checksum: string): ResolvedLocalFile {
  return {
    path: "/tmp/.agents/project.md",
    manifestPath: ".agents/project.md",
    content: "note\n",
    sourceChecksum: checksum,
    position: "after",
    optional: false,
  };
}

function lockfileWithLocal(checksum: string): Lockfile {
  return {
    prayfile_lock: "1",
    spec: "prayfile-1",
    generated_by: "test",
    manifest_hash: "sha256:abc",
    source: [],
    package: [],
    target: [],
    managed_span: [
      {
        id: "aaaa1111",
        target: "AGENTS.md",
        open_line: 1,
        close_line: 3,
        ideal_checksum: checksum,
        package: LOCAL_EMBED_PACKAGE,
        export: ".agents/project.md",
        source_checksum: checksum,
        silenced: false,
      },
    ],
    provisioned: [],
  };
}

function projectWithLocal(checksum: string): ResolvedProject {
  return {
    manifestPath: "/tmp/Prayfile",
    projectRoot: "/tmp",
    manifest: {
      prayfileVersion: "1",
      sources: [],
      targets: [],
      packages: [],
      local: [],
      symbols: {},
      render: {
        mode: "managed",
        conflict: "fail",
        churn: "minimal",
        header: true,
      },
    },
    manifestHash: "sha256:abc",
    packages: [],
    localFiles: [localFile(checksum)],
    sourceRevisions: new Map(),
    sourceHostKeys: new Map(),
  };
}

describe("local apply report", () => {
  it("prints checked when the local checksum is unchanged", () => {
    const lines = localSummaryLines(
      lockfileWithLocal("sha256:old"),
      projectWithLocal("sha256:old"),
    );
    assert.equal(lines[0]?.includes("checked"), true);
  });

  it("lists local checksum drift for outdated", () => {
    const lines = outdatedLocalLines(
      lockfileWithLocal("sha256:old"),
      projectWithLocal("sha256:new"),
    );
    assert.equal(lines[0]?.includes("sha256:old -> sha256:new"), true);
  });
});
