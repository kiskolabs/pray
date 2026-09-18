use crate::publish_remote::ManifestPublishRemote;
use crate::{PrayError, PrayResult};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishDestination {
    pub name: String,
    pub root: Option<PathBuf>,
    pub server: Option<String>,
    pub packages: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct PublishCliDest {
    pub to: Vec<String>,
    pub roots: Vec<PathBuf>,
    pub servers: Vec<String>,
}

pub fn resolve_publish_destinations(
    remotes: &[ManifestPublishRemote],
    cli: &PublishCliDest,
    project_root: &Path,
) -> PrayResult<Vec<PublishDestination>> {
    if !cli.to.is_empty() && (!cli.roots.is_empty() || !cli.servers.is_empty()) {
        return Err(PrayError::Usage(
            "publish --to cannot be combined with --root or --server".to_string(),
        ));
    }
    if remotes.is_empty() {
        return resolve_undeclared_destinations(cli);
    }
    let selected = select_remotes(remotes, cli, project_root)?;
    Ok(selected
        .into_iter()
        .map(|remote| PublishDestination {
            name: remote.name.clone(),
            root: remote.path.as_ref().map(|path| project_root.join(path)),
            server: remote.url.clone(),
            packages: remote.packages.clone(),
        })
        .collect())
}

pub fn resolve_path_remote(
    remotes: &[ManifestPublishRemote],
    to: Option<&str>,
    root: Option<&Path>,
    project_root: &Path,
) -> PrayResult<PathBuf> {
    let path_remotes: Vec<&ManifestPublishRemote> = remotes
        .iter()
        .filter(|remote| remote.path.is_some())
        .collect();
    if let Some(name) = to {
        let remote = remotes
            .iter()
            .find(|remote| remote.name == name)
            .ok_or_else(|| PrayError::Usage(format!("unknown publish remote: {name}")))?;
        let path = remote.path.as_ref().ok_or_else(|| {
            PrayError::Usage(format!(
                "publish remote {name} is a URL; yank, serve, and token need a path remote"
            ))
        })?;
        return Ok(project_root.join(path));
    }
    if remotes.is_empty() {
        return root
            .map(Path::to_path_buf)
            .ok_or_else(|| PrayError::Usage("requires --root PATH".to_string()));
    }
    if let Some(root) = root {
        let matched = remotes.iter().find(|remote| {
            remote
                .path
                .as_ref()
                .is_some_and(|path| root_matches(root, path, project_root))
        });
        if matched.is_none() {
            return Err(PrayError::Usage(format!(
                "path {} is not a declared publish remote",
                root.display()
            )));
        }
        return Ok(root.to_path_buf());
    }
    if path_remotes.len() == 1 {
        if let Some(path) = &path_remotes[0].path {
            return Ok(project_root.join(path));
        }
    }
    Err(PrayError::Usage(
        "say which path remote with --to NAME or --root PATH".to_string(),
    ))
}

fn resolve_undeclared_destinations(cli: &PublishCliDest) -> PrayResult<Vec<PublishDestination>> {
    if !cli.to.is_empty() {
        return Err(PrayError::Usage(
            "Prayfile has no publish remotes; pass --root PATH or --server URL".to_string(),
        ));
    }
    if cli.roots.is_empty() && cli.servers.is_empty() {
        return Err(PrayError::Unsupported(
            "publish requires at least one --root PATH or --server URL".to_string(),
        ));
    }
    let mut dests = Vec::new();
    for root in &cli.roots {
        dests.push(PublishDestination {
            name: root.display().to_string(),
            root: Some(root.clone()),
            server: None,
            packages: Vec::new(),
        });
    }
    for server in &cli.servers {
        dests.push(PublishDestination {
            name: server.clone(),
            root: None,
            server: Some(server.clone()),
            packages: Vec::new(),
        });
    }
    Ok(dests)
}

fn select_remotes<'a>(
    remotes: &'a [ManifestPublishRemote],
    cli: &PublishCliDest,
    project_root: &Path,
) -> PrayResult<Vec<&'a ManifestPublishRemote>> {
    if !cli.to.is_empty() {
        let mut selected = Vec::new();
        for name in &cli.to {
            let remote = remotes
                .iter()
                .find(|remote| remote.name == *name)
                .ok_or_else(|| PrayError::Usage(format!("unknown publish remote: {name}")))?;
            selected.push(remote);
        }
        return Ok(selected);
    }
    if cli.roots.is_empty() && cli.servers.is_empty() {
        return Ok(remotes.iter().collect());
    }
    let mut selected = Vec::new();
    for root in &cli.roots {
        let remote = remotes
            .iter()
            .find(|remote| {
                remote
                    .path
                    .as_ref()
                    .is_some_and(|path| root_matches(root, path, project_root))
            })
            .ok_or_else(|| {
                PrayError::Usage(format!(
                    "path {} is not a declared publish remote",
                    root.display()
                ))
            })?;
        selected.push(remote);
    }
    for server in &cli.servers {
        let remote = remotes
            .iter()
            .find(|remote| remote.url.as_deref() == Some(server.as_str()))
            .ok_or_else(|| {
                PrayError::Usage(format!("server {server} is not a declared publish remote"))
            })?;
        selected.push(remote);
    }
    Ok(selected)
}

pub(crate) fn root_matches(cli_root: &Path, remote_path: &str, project_root: &Path) -> bool {
    let declared = project_root.join(remote_path);
    if cli_root == declared || cli_root == Path::new(remote_path) {
        return true;
    }
    strip_dot_slash(&cli_root.to_string_lossy()) == strip_dot_slash(remote_path)
}

fn strip_dot_slash(value: &str) -> String {
    value
        .trim()
        .trim_start_matches("./")
        .trim_end_matches('/')
        .replace('\\', "/")
}
