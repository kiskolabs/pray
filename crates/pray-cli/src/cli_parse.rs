use crate::Command;
use pray_core::cli_suggest::unknown_command_message;
use pray_core::{PrayError, PrayResult};
use std::path::PathBuf;

pub(crate) fn parse_command(arguments: Vec<String>) -> PrayResult<Command> {
    let check = arguments.iter().any(|argument| argument == "--check");
    let strict = arguments.iter().any(|argument| argument == "--strict");
    let semantic = arguments.iter().any(|argument| argument == "--semantic");
    let mut iter = arguments.into_iter();
    let command = iter
        .next()
        .ok_or_else(|| PrayError::Usage("pray requires a command; run pray --help".to_string()))?;
    match command.as_str() {
        "manifest" => Ok(Command::Manifest),
        "init" => {
            let mut targets = Vec::new();
            while let Some(argument) = iter.next() {
                if argument == "--targets" {
                    if let Some(value) = iter.next() {
                        targets = value
                            .split(',')
                            .map(|entry| entry.trim().to_string())
                            .filter(|entry| !entry.is_empty())
                            .collect();
                    }
                }
            }
            Ok(Command::Init { targets })
        }
        "prayer" => parse_namespaced_init_command("prayer", iter, Command::PrayerInit),
        "repo" => parse_namespaced_init_command("repo", iter, Command::RepoInit),
        "install" => {
            let mut locked = false;
            let mut frozen = false;
            let mut offline = false;
            for argument in iter {
                match argument.as_str() {
                    "--locked" => locked = true,
                    "--frozen" => {
                        locked = true;
                        frozen = true;
                    }
                    "--offline" => offline = true,
                    other if other.starts_with("--") => {
                        return Err(PrayError::Unsupported(format!(
                            "unknown install flag: {other}"
                        )))
                    }
                    other => {
                        return Err(PrayError::Unsupported(format!(
                            "unexpected install argument: {other}"
                        )))
                    }
                }
            }
            Ok(Command::Install {
                locked,
                frozen,
                offline,
            })
        }
        "add" => parse_add_command(iter),
        "remove" => parse_remove_command(iter),
        "update" => parse_update_command(iter),
        "unlock" => parse_unlock_command(iter),
        "render" => Ok(Command::Render { check }),
        "plan" => parse_plan_command(iter),
        "apply" => Ok(Command::Apply),
        "verify" => Ok(Command::Verify { strict }),
        "drift" => Ok(Command::Drift { semantic }),
        "format" | "fmt" => Ok(Command::Format),
        "package" => Ok(Command::Package),
        "publish" => parse_publish_command(iter),
        "login" => parse_login_command(iter),
        #[cfg(feature = "auth")]
        "serve" => parse_serve_command(iter),
        #[cfg(not(feature = "auth"))]
        "serve" => Err(PrayError::Usage("unknown command: serve".to_string())),
        "confess" => parse_confess_command(iter),
        "list" => Ok(Command::List),
        "outdated" => parse_outdated_command(iter),
        "explain" => parse_explain_command(iter),
        "vendor" => Ok(Command::Vendor),
        "clean" => Ok(Command::Clean),
        "tree" => Ok(Command::Tree),
        "sync" => parse_sync_command(iter),
        "trust" => {
            let arguments: Vec<String> = iter.collect();
            Ok(Command::Trust { arguments })
        }
        "upgrade" => Ok(Command::Upgrade),
        "version" | "-V" | "--version" => Ok(Command::Version),
        other => Err(PrayError::Usage(unknown_command_message(other))),
    }
}

fn parse_namespaced_init_command(
    namespace: &str,
    mut arguments: std::vec::IntoIter<String>,
    command: Command,
) -> PrayResult<Command> {
    match arguments.next() {
        Some(subcommand) if subcommand == "init" => {
            if let Some(argument) = arguments.next() {
                return Err(PrayError::Unsupported(format!(
                    "unexpected {namespace} argument: {argument}"
                )));
            }
            Ok(command)
        }
        Some(other) => Err(PrayError::Unsupported(format!(
            "unknown {namespace} command: {other}"
        ))),
        None => Err(PrayError::Unsupported(format!("{namespace} requires init"))),
    }
}

fn parse_add_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
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

fn parse_remove_command(arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
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

fn parse_update_command(arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
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

fn parse_plan_command(arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
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

fn parse_outdated_command(arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
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

fn parse_unlock_command(arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
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

fn parse_publish_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
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

fn parse_login_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
    let mut servers = Vec::new();
    let mut email = None;
    let mut credential_id = None;
    let mut passkey_key = None;
    let mut public_key = None;
    let mut ssh_agent = false;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--server" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "login requires a URL after --server".to_string(),
                    ));
                };
                servers.push(value);
            }
            "--email" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "login requires an email after --email".to_string(),
                    ));
                };
                email = Some(value);
            }
            "--credential-id" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "login requires a credential id after --credential-id".to_string(),
                    ));
                };
                credential_id = Some(value);
            }
            "--passkey-key" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "login requires a path after --passkey-key".to_string(),
                    ));
                };
                passkey_key = Some(PathBuf::from(value));
            }
            "--public-key" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "login requires a path after --public-key".to_string(),
                    ));
                };
                public_key = Some(PathBuf::from(value));
            }
            "--ssh-agent" => ssh_agent = true,
            other if other.starts_with("--") => {
                return Err(PrayError::Unsupported(format!(
                    "unknown login flag: {other}"
                )))
            }
            other => {
                return Err(PrayError::Unsupported(format!(
                    "unexpected login argument: {other}"
                )))
            }
        }
    }
    if servers.is_empty() {
        return Err(PrayError::Unsupported(
            "login requires at least one --server URL".to_string(),
        ));
    }
    let email = email
        .ok_or_else(|| PrayError::Unsupported("login requires --email ADDRESS".to_string()))?;
    if passkey_key.is_some() == ssh_agent || (passkey_key.is_none() && public_key.is_none()) {
        return Err(PrayError::Unsupported(
            "login requires exactly one authentication mode".to_string(),
        ));
    }
    if passkey_key.is_some() && credential_id.is_none() {
        return Err(PrayError::Unsupported(
            "passkey login requires --credential-id".to_string(),
        ));
    }
    if ssh_agent && public_key.is_none() {
        return Err(PrayError::Unsupported(
            "ssh-agent login requires --public-key".to_string(),
        ));
    }
    Ok(Command::Login {
        servers,
        email,
        credential_id,
        passkey_key,
        public_key,
        ssh_agent,
    })
}

fn parse_serve_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
    let mut root = PathBuf::from(".");
    let mut host = "127.0.0.1".to_string();
    let mut port = 7429u16;
    let mut stdio = false;
    let mut allow_open_push = false;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--root" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "serve requires a path after --root".to_string(),
                    ));
                };
                root = PathBuf::from(value);
            }
            "--host" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "serve requires a host after --host".to_string(),
                    ));
                };
                host = value;
            }
            "--port" => {
                let Some(value) = arguments.next() else {
                    return Err(PrayError::Unsupported(
                        "serve requires a port after --port".to_string(),
                    ));
                };
                port = value
                    .parse::<u16>()
                    .map_err(|error| PrayError::Unsupported(error.to_string()))?;
            }
            "--stdio" => stdio = true,
            "--allow-open-push" => allow_open_push = true,
            other if other.starts_with("--") => {
                return Err(PrayError::Unsupported(format!(
                    "unknown serve flag: {other}"
                )))
            }
            other => {
                return Err(PrayError::Unsupported(format!(
                    "unexpected serve argument: {other}"
                )))
            }
        }
    }
    Ok(Command::Serve {
        root,
        host,
        port,
        stdio,
        allow_open_push,
    })
}

fn parse_sync_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
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

fn parse_confess_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
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

fn parse_explain_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
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
