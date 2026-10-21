use std::path::Path;
use std::process::Command;

// Mirrors git's own sanitize_repo_env: `git rev-parse --local-env-vars` minus
// GIT_CONFIG_PARAMETERS and GIT_CONFIG_COUNT. Git exports these to hooks, so
// pray started from a hook would otherwise run every cache command against the
// host repository. The two config variables pass through, as they do when git
// enters a submodule, because callers such as CI jobs use them to hand git
// credentials or URL rewrites, and they cannot select a repository.
const REPOSITORY_LOCAL_VARIABLES: [&str; 13] = [
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
];

/// Remove git repository-local variables that hooks export into child processes.
pub fn clear_repository_local_env(command: &mut Command) {
    for name in REPOSITORY_LOCAL_VARIABLES {
        command.env_remove(name);
    }
}

// Credential, SSH, proxy and global config variables still pass through, so
// the environment is pruned rather than cleared.
pub(crate) fn git_command(repository: &Path) -> Command {
    let mut command = Command::new(git_program());
    command.current_dir(repository);
    clear_repository_local_env(&mut command);
    command
}

fn git_program() -> &'static str {
    [
        "/usr/bin/git",
        "/opt/homebrew/bin/git",
        "/usr/local/bin/git",
        "git",
    ]
    .into_iter()
    .find(|candidate| Path::new(candidate).exists())
    .unwrap_or("git")
}
