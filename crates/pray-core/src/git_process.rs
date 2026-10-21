use std::path::Path;
use std::process::Command;

// Mirrors `git rev-parse --local-env-vars`. Git exports these to hooks, so pray
// started from a hook would otherwise run every cache command against the host
// repository. GIT_CONFIG_KEY_n/GIT_CONFIG_VALUE_n are left alone: git reads them
// only when GIT_CONFIG_COUNT is set, and callers that set a count also set their
// own pairs.
const REPOSITORY_LOCAL_VARIABLES: [&str; 15] = [
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_CONFIG",
    "GIT_CONFIG_PARAMETERS",
    "GIT_CONFIG_COUNT",
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

// Credential, SSH, proxy and global config variables still pass through, so
// the environment is pruned rather than cleared.
pub(crate) fn git_command(repository: &Path) -> Command {
    let mut command = Command::new(git_program());
    command.current_dir(repository);
    for name in REPOSITORY_LOCAL_VARIABLES {
        command.env_remove(name);
    }
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
