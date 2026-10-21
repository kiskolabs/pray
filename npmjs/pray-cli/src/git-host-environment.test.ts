import assert from "node:assert/strict";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { describe, it } from "node:test";
import { ensureGitRepository } from "./git/cache.js";
import {
  commitAll,
  type PinnedShallowCache,
  pinnedShallowCache,
  runGit,
} from "./git-catalog.support.js";

interface HostRepository {
  path: string;
  gitDir: string;
}

interface HostState {
  originUrl: string;
  refs: string;
  index: Buffer;
}

describe("git commands started from a hook", () => {
  it("leave the host repository alone when GIT_DIR is set", () => {
    assertHostUntouchedWith((host) => ({ GIT_DIR: host.gitDir }));
  });

  it("leave the host index alone when GIT_INDEX_FILE is set", () => {
    assertHostUntouchedWith((host) => ({
      GIT_INDEX_FILE: join(host.gitDir, "index"),
    }));
  });

  it("leave the host repository alone when GIT_WORK_TREE is set", () => {
    assertHostUntouchedWith((host) => ({
      GIT_DIR: host.gitDir,
      GIT_WORK_TREE: host.path,
      GIT_INDEX_FILE: join(host.gitDir, "index"),
    }));
  });

  it("leave the host HEAD and branches alone on refresh", () => {
    const fixture = pinnedShallowCache();
    try {
      const host = hostRepository(fixture);
      const before = hostState(host);
      const result = withCallerEnv({ GIT_DIR: host.gitDir }, () =>
        ensureGitRepository(fixture.root, fixture.cloneUrl, true),
      );
      assertHostStateUnchanged(host, before);
      assert.equal(
        result.revision,
        runGit(fixture.origin, "rev-parse", "HEAD").trim(),
      );
    } finally {
      fixture.cleanup();
    }
  });

  it("apply git config the caller passes in the environment", () => {
    const fixture = pinnedShallowCache();
    try {
      const alias = "file:///pray-caller-config-alias";
      const result = withCallerEnv(
        {
          GIT_CONFIG_COUNT: "1",
          GIT_CONFIG_KEY_0: `url.${fixture.cloneUrl}.insteadOf`,
          GIT_CONFIG_VALUE_0: alias,
        },
        () => ensureGitRepository(fixture.root, alias, false),
      );
      assert.equal(
        result.revision,
        runGit(fixture.origin, "rev-parse", "HEAD").trim(),
      );
    } finally {
      fixture.cleanup();
    }
  });

  it("repairs a broken cache without touching the host repository", () => {
    const fixture = pinnedShallowCache();
    try {
      const hostRemote = "https://example.com/host.git";
      writeFileSync(join(fixture.root, "README.md"), "host\n");
      runGit(fixture.root, "init", "--template=", "-b", "main");
      runGit(fixture.root, "remote", "add", "origin", hostRemote);
      rmSync(join(fixture.db, "objects"), { recursive: true, force: true });
      const result = ensureGitRepository(
        fixture.root,
        fixture.cloneUrl,
        false,
        fixture.pinned,
      );
      assert.ok(existsSync(join(fixture.db, "HEAD")));
      assert.ok(existsSync(join(fixture.db, "objects")));
      assert.equal(result.revision, fixture.pinned);
      assert.equal(
        runGit(fixture.root, "remote", "get-url", "origin").trim(),
        hostRemote,
      );
    } finally {
      fixture.cleanup();
    }
  });
});

function assertHostUntouchedWith(
  callerEnv: (host: HostRepository) => Record<string, string>,
): void {
  const fixture = pinnedShallowCache();
  try {
    const host = hostRepository(fixture);
    const before = hostState(host);
    const result = withCallerEnv(callerEnv(host), () =>
      ensureGitRepository(
        fixture.root,
        fixture.cloneUrl,
        false,
        fixture.pinned,
      ),
    );
    assertHostStateUnchanged(host, before);
    assert.equal(result.revision, fixture.pinned);
  } finally {
    fixture.cleanup();
  }
}

function withCallerEnv<T>(variables: Record<string, string>, run: () => T): T {
  const previous = new Map(
    Object.keys(variables).map((name) => [name, process.env[name]]),
  );
  Object.assign(process.env, variables);
  try {
    return run();
  } finally {
    for (const [name, value] of previous) {
      if (value === undefined) delete process.env[name];
      else process.env[name] = value;
    }
  }
}

function hostRepository(fixture: PinnedShallowCache): HostRepository {
  const remote = join(fixture.root, "host-remote.git");
  runGit(fixture.root, "init", "--bare", "--template=", remote);
  const path = join(fixture.root, "host");
  mkdirSync(path, { recursive: true });
  writeFileSync(join(path, "README.md"), "host\n");
  runGit(path, "init", "--template=", "-b", "main");
  commitAll(path, "host");
  runGit(path, "remote", "add", "origin", remote);
  runGit(path, "push", "origin", "main");
  runGit(path, "fetch", "origin");
  return { path, gitDir: join(path, ".git") };
}

function hostState(host: HostRepository): HostState {
  return {
    originUrl: runGit(host.path, "remote", "get-url", "origin").trim(),
    refs: runGit(host.path, "show-ref", "--head").trim(),
    index: readFileSync(join(host.gitDir, "index")),
  };
}

function assertHostStateUnchanged(
  host: HostRepository,
  before: HostState,
): void {
  const after = hostState(host);
  const changes: string[] = [];
  if (after.originUrl !== before.originUrl) {
    changes.push(`origin URL: ${before.originUrl} -> ${after.originUrl}`);
  }
  if (after.refs !== before.refs) {
    changes.push(`HEAD or refs:\n${before.refs}\n->\n${after.refs}`);
  }
  if (!after.index.equals(before.index)) {
    changes.push("index");
  }
  assert.deepEqual(
    changes,
    [],
    `host repository changed:\n${changes.join("\n")}`,
  );
}
