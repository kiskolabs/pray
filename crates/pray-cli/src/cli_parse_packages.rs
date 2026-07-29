use crate::Command;
use pray_core::{PrayError, PrayResult};

pub(crate) fn parse_add_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
    let mut name = None;
    let mut constraint = None;
    let mut path = None;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--path" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "add requires a path after --path".to_string(),
                    ));
                };
                path = Some(value);
            }
            other if other.starts_with("--") => {
                return Err(PrayError::Unsupported(format!("unknown add flag: {other}")))
            }
            other => {
                if name.is_none() {
                    name = Some(other.to_string());
                } else if constraint.is_none() {
                    constraint = Some(other.to_string());
                } else {
                    return Err(PrayError::Unsupported(format!(
                        "unexpected add argument: {other}"
                    )));
                }
            }
        }
    }
    let name =
        name.ok_or_else(|| PrayError::Unsupported("add requires a package name".to_string()))?;
    Ok(Command::Add {
        name,
        constraint,
        path,
    })
}

pub(crate) fn parse_remove_command(arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
    let mut name = None;
    for argument in arguments {
        match argument.as_str() {
            other if other.starts_with("--") => {
                return Err(PrayError::Unsupported(format!(
                    "unknown remove flag: {other}"
                )))
            }
            other => {
                if name.is_none() {
                    name = Some(other.to_string());
                } else {
                    return Err(PrayError::Unsupported(format!(
                        "unexpected remove argument: {other}"
                    )));
                }
            }
        }
    }
    let name =
        name.ok_or_else(|| PrayError::Unsupported("remove requires a package name".to_string()))?;
    Ok(Command::Remove { name })
}

pub(crate) fn parse_update_command(arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
    let mut package = None;
    let mut major = false;
    let mut latest = false;
    let mut dry_run = false;
    let mut json = false;
    for argument in arguments {
        match argument.as_str() {
            "--major" => major = true,
            "--latest" => latest = true,
            "--dry-run" => dry_run = true,
            "--json" => json = true,
            other if other.starts_with("--") => {
                return Err(PrayError::Unsupported(format!(
                    "unknown update flag: {other}"
                )))
            }
            other => {
                if package.is_none() {
                    package = Some(other.to_string());
                } else {
                    return Err(PrayError::Unsupported(format!(
                        "unexpected update argument: {other}"
                    )));
                }
            }
        }
    }
    Ok(Command::Update {
        package,
        major,
        latest,
        dry_run,
        json,
    })
}

pub(crate) fn parse_plan_command(arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
    let mut remote = false;
    for argument in arguments {
        match argument.as_str() {
            "--remote" => remote = true,
            other => {
                return Err(PrayError::Unsupported(format!(
                    "unknown plan flag: {other}"
                )))
            }
        }
    }
    Ok(Command::Plan { remote })
}

pub(crate) fn parse_outdated_command(arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
    let mut remote = false;
    for argument in arguments {
        match argument.as_str() {
            "--remote" => remote = true,
            other => {
                return Err(PrayError::Unsupported(format!(
                    "unknown outdated flag: {other}"
                )))
            }
        }
    }
    Ok(Command::Outdated { remote })
}

pub(crate) fn parse_unlock_command(arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
    let mut package = None;
    for argument in arguments {
        match argument.as_str() {
            other if other.starts_with("--") => {
                return Err(PrayError::Unsupported(format!(
                    "unknown unlock flag: {other}"
                )))
            }
            other => {
                if package.is_none() {
                    package = Some(other.to_string());
                } else {
                    return Err(PrayError::Unsupported(format!(
                        "unexpected unlock argument: {other}"
                    )));
                }
            }
        }
    }
    let package = package
        .ok_or_else(|| PrayError::Unsupported("unlock requires a package name".to_string()))?;
    Ok(Command::Unlock { package })
}

pub(crate) fn parse_explain_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
    let package = arguments
        .next()
        .ok_or_else(|| PrayError::Unsupported("explain requires a package name".to_string()))?;
    if let Some(argument) = arguments.next() {
        return Err(PrayError::Unsupported(format!(
            "unexpected explain argument: {argument}"
        )));
    }
    Ok(Command::Explain { package })
}
