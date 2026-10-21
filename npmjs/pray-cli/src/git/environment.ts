// Mirrors git's own sanitize_repo_env: `git rev-parse --local-env-vars` minus
// GIT_CONFIG_PARAMETERS and GIT_CONFIG_COUNT. Git exports these to hooks, so
// pray started from a hook would otherwise run every cache command against the
// host repository. The two config variables pass through, as they do when git
// enters a submodule, because callers such as CI jobs use them to hand git
// credentials or URL rewrites, and they cannot select a repository.
const REPOSITORY_LOCAL_VARIABLES = [
  "GIT_ALTERNATE_OBJECT_DIRECTORIES",
  "GIT_CONFIG",
  "GIT_OBJECT_DIRECTORY",
  "GIT_DIR",
  "GIT_WORK_TREE",
  "GIT_IMPLICIT_WORK_TREE",
  "GIT_GRAFT_FILE",
  "GIT_INDEX_FILE",
  "GIT_NO_REPLACE_OBJECTS",
  "GIT_REPLACE_REF_BASE",
  "GIT_PREFIX",
  "GIT_SHALLOW_FILE",
  "GIT_COMMON_DIR",
] as const;

// Credential, SSH, proxy and global config variables still pass through, so
// the environment is pruned rather than cleared.
export function gitEnvironment(
  extra: Record<string, string> = {},
): NodeJS.ProcessEnv {
  const environment: NodeJS.ProcessEnv = { ...process.env, ...extra };
  for (const name of REPOSITORY_LOCAL_VARIABLES) {
    delete environment[name];
  }
  return environment;
}
