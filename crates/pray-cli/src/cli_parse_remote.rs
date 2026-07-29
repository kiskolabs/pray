use crate::Command;
use pray_core::{PrayError, PrayResult};
use std::path::PathBuf;

pub(crate) fn parse_search_command(
    mut arguments: std::vec::IntoIter<String>,
) -> PrayResult<Command> {
    let mut query = None;
    let mut source = None;
    let mut root = None;
    let mut url = None;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--source" => {
                source = Some(arguments.next().ok_or_else(|| {
                    PrayError::Unsupported("search requires a name after --source".into())
                })?);
            }
            "--root" => {
                root = Some(PathBuf::from(arguments.next().ok_or_else(|| {
                    PrayError::Unsupported("search requires a path after --root".into())
                })?));
            }
            "--url" => {
                url = Some(arguments.next().ok_or_else(|| {
                    PrayError::Unsupported("search requires a URL after --url".into())
                })?);
            }
            other if other.starts_with("--") => {
                return Err(PrayError::Unsupported(format!(
                    "unknown search flag: {other}"
                )))
            }
            other => {
                if query.is_none() {
                    query = Some(other.to_string());
                } else {
                    return Err(PrayError::Unsupported(format!(
                        "unexpected search argument: {other}"
                    )));
                }
            }
        }
    }
    let query =
        query.ok_or_else(|| PrayError::Unsupported("search requires a query".to_string()))?;
    Ok(Command::Search {
        query,
        source,
        root,
        url,
    })
}

pub(crate) fn parse_yank_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
    let mut package = None;
    let mut version = None;
    let mut root = None;
    let mut undo = false;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--root" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "yank requires a path after --root".to_string(),
                    ));
                };
                root = Some(PathBuf::from(value));
            }
            "--undo" => undo = true,
            other if other.starts_with("--") => {
                return Err(PrayError::Unsupported(format!(
                    "unknown yank flag: {other}"
                )))
            }
            other => {
                if package.is_none() {
                    package = Some(other.to_string());
                } else if version.is_none() {
                    version = Some(other.to_string());
                } else {
                    return Err(PrayError::Unsupported(format!(
                        "unexpected yank argument: {other}"
                    )));
                }
            }
        }
    }
    let package = package
        .ok_or_else(|| PrayError::Unsupported("yank requires a package name".to_string()))?;
    let version =
        version.ok_or_else(|| PrayError::Unsupported("yank requires a version".to_string()))?;
    let root =
        root.ok_or_else(|| PrayError::Unsupported("yank requires --root PATH".to_string()))?;
    Ok(Command::Yank {
        package,
        version,
        root,
        undo,
    })
}

pub(crate) fn parse_publish_command(
    mut arguments: std::vec::IntoIter<String>,
) -> PrayResult<Command> {
    let mut roots = Vec::new();
    let mut servers = Vec::new();
    let mut signing_key = None;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--root" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "publish requires a path after --root".to_string(),
                    ));
                };
                roots.push(PathBuf::from(value));
            }
            "--server" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "publish requires a URL after --server".to_string(),
                    ));
                };
                servers.push(value);
            }
            "--signing-key" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "publish requires a path after --signing-key".to_string(),
                    ));
                };
                signing_key = Some(PathBuf::from(value));
            }
            other if other.starts_with("--") => {
                return Err(PrayError::Unsupported(format!(
                    "unknown publish flag: {other}"
                )))
            }
            other => {
                return Err(PrayError::Unsupported(format!(
                    "unexpected publish argument: {other}"
                )))
            }
        }
    }
    if roots.is_empty() && servers.is_empty() {
        return Err(PrayError::Unsupported(
            "publish requires at least one --root PATH or --server URL".to_string(),
        ));
    }
    Ok(Command::Publish {
        roots,
        servers,
        signing_key,
    })
}

pub(crate) fn parse_sync_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
    let mut root = PathBuf::from(".");
    let mut peers = Vec::new();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--root" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "sync requires a path after --root".to_string(),
                    ));
                };
                root = PathBuf::from(value);
            }
            "--peer" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "sync requires a URL after --peer".to_string(),
                    ));
                };
                peers.push(value);
            }
            other if other.starts_with("--") => {
                return Err(PrayError::Unsupported(format!(
                    "unknown sync flag: {other}"
                )))
            }
            other => {
                return Err(PrayError::Unsupported(format!(
                    "unexpected sync argument: {other}"
                )))
            }
        }
    }
    Ok(Command::Sync { root, peers })
}

pub(crate) fn parse_confess_command(
    mut arguments: std::vec::IntoIter<String>,
) -> PrayResult<Command> {
    let mut package = None;
    let mut from_lock = None;
    let mut version = None;
    let mut accepted = false;
    let mut rejected = false;
    let mut note = None;
    let mut url = None;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--from-lock" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "confess requires a lockfile span id after --from-lock".to_string(),
                    ));
                };
                from_lock = Some(value);
            }
            "--version" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "confess requires a version after --version".to_string(),
                    ));
                };
                version = Some(value);
            }
            "--note" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "confess requires a note after --note".to_string(),
                    ));
                };
                note = Some(value);
            }
            "--url" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "confess requires a URL after --url".to_string(),
                    ));
                };
                url = Some(value);
            }
            "--accepted" => accepted = true,
            "--rejected" => rejected = true,
            other if other.starts_with("--") => {
                return Err(PrayError::Unsupported(format!(
                    "unknown confess flag: {other}"
                )))
            }
            other => {
                if package.is_none() && from_lock.is_none() {
                    package = Some(other.to_string());
                } else {
                    return Err(PrayError::Unsupported(format!(
                        "unexpected confess argument: {other}"
                    )));
                }
            }
        }
    }
    if package.is_some() == from_lock.is_some() {
        return Err(PrayError::Unsupported(
            "confess requires exactly one of a package name or --from-lock".to_string(),
        ));
    }
    if accepted == rejected {
        return Err(PrayError::Unsupported(
            "confess requires exactly one of --accepted or --rejected".to_string(),
        ));
    }
    Ok(Command::Confess {
        package,
        from_lock,
        version,
        accepted,
        rejected,
        note,
        url,
    })
}
