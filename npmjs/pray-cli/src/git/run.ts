import { spawnSync } from "node:child_process";
import { PrayError } from "../errors.js";

function spawnGit(repository: string, argumentsList: string[]) {
  return spawnSync(
    "git",
    ["-c", "protocol.file.allow=always", "-C", repository, ...argumentsList],
    {
      encoding: "utf8" as const,
      env: { ...process.env, GIT_TERMINAL_PROMPT: "0" },
      input: "",
    },
  );
}

export function tryRunGit(
  repository: string,
  ...argumentsList: string[]
): boolean {
  const result = spawnGit(repository, argumentsList);
  if ((result.error as NodeJS.ErrnoException | undefined)?.code === "ENOENT") {
    throw PrayError.unsupported("git is required for git sources");
  }
  return result.status === 0;
}

export function runGit(repository: string, ...argumentsList: string[]): void {
  const result = spawnGit(repository, argumentsList);
  if ((result.error as NodeJS.ErrnoException | undefined)?.code === "ENOENT") {
    throw PrayError.unsupported("git is required for git sources");
  }
  if (result.status !== 0) {
    throw PrayError.resolution(
      commandError(
        `git ${argumentsList.join(" ")}`,
        result.stderr ?? result.stdout ?? "",
      ),
    );
  }
}

export function runGitCapture(
  repository: string,
  ...argumentsList: string[]
): string {
  const result = spawnGit(repository, argumentsList);
  if (result.status !== 0) {
    throw PrayError.resolution(
      commandError(
        `git ${argumentsList.join(" ")}`,
        result.stderr ?? result.stdout ?? "",
      ),
    );
  }
  return result.stdout ?? "";
}

export function commandError(program: string, output: string): string {
  const message = output.trim();
  return message.length === 0
    ? `${program} failed`
    : `${program} failed: ${message}`;
}
