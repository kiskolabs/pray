use std::path::PathBuf;

pub(crate) enum Command {
    Manifest,
    Init {
        targets: Vec<String>,
    },
    PrayerInit,
    RepoInit,
    Install {
        locked: bool,
        frozen: bool,
        offline: bool,
    },
    Add {
        name: String,
        constraint: Option<String>,
        path: Option<String>,
    },
    Remove {
        name: String,
    },
    Update {
        package: Option<String>,
        major: bool,
        latest: bool,
        dry_run: bool,
        json: bool,
    },
    Unlock {
        package: String,
    },
    Render {
        check: bool,
    },
    Plan {
        remote: bool,
    },
    Apply,
    Verify {
        strict: bool,
    },
    Drift {
        semantic: bool,
    },
    Format,
    Package,
    Publish {
        roots: Vec<PathBuf>,
        servers: Vec<String>,
        signing_key: Option<PathBuf>,
    },
    Login {
        servers: Vec<String>,
        email: String,
        credential_id: Option<String>,
        passkey_key: Option<PathBuf>,
        public_key: Option<PathBuf>,
        ssh_agent: bool,
    },
    #[cfg(feature = "auth")]
    Serve {
        root: PathBuf,
        host: String,
        port: u16,
        stdio: bool,
        allow_open_push: bool,
    },
    Confess {
        package: Option<String>,
        from_lock: Option<String>,
        version: Option<String>,
        accepted: bool,
        rejected: bool,
        note: Option<String>,
        url: Option<String>,
    },
    List,
    Outdated {
        remote: bool,
    },
    Explain {
        package: String,
    },
    Vendor,
    Clean,
    Tree,
    Sync {
        root: PathBuf,
        peers: Vec<String>,
    },
    Trust {
        arguments: Vec<String>,
    },
    Upgrade,
    Version,
}
