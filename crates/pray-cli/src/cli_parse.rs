use crate::cli_parse_auth::parse_login_command;
#[cfg(feature = "auth")]
use crate::cli_parse_auth::parse_serve_command;
use crate::cli_parse_packages::{
    parse_add_command, parse_explain_command, parse_outdated_command, parse_plan_command,
    parse_remove_command, parse_unlock_command, parse_update_command,
};
use crate::cli_parse_remote::{
    parse_confess_command, parse_publish_command, parse_search_command, parse_sync_command,
    parse_yank_command,
};
use crate::Command;
use pray_core::cli_suggest::unknown_command_message;
use pray_core::{PrayError, PrayResult};

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
            let mut strict = false;
            for argument in iter {
                match argument.as_str() {
                    "--locked" => locked = true,
                    "--frozen" => {
                        locked = true;
                        frozen = true;
                    }
                    "--offline" => offline = true,
                    "--strict" => strict = true,
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
                strict,
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
        "yank" => parse_yank_command(iter),
        "search" => parse_search_command(iter),
        #[cfg(feature = "auth")]
        "token" => Ok(Command::Token {
            arguments: iter.collect(),
        }),
        #[cfg(not(feature = "auth"))]
        "token" => Err(PrayError::Usage("unknown command: token".to_string())),
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
        "completion" => parse_completion_command(iter),
        other => Err(PrayError::Usage(unknown_command_message(other))),
    }
}

fn parse_completion_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
    let shell = arguments.next().ok_or_else(|| {
        PrayError::Usage("completion requires bash, zsh, or fish\nSee 'pray --help'.".to_string())
    })?;
    if arguments.next().is_some() {
        return Err(PrayError::Usage(
            "completion accepts one shell argument: bash, zsh, or fish\nSee 'pray --help'."
                .to_string(),
        ));
    }
    match shell.as_str() {
        "bash" | "zsh" | "fish" => Ok(Command::Completion { shell }),
        _ => Err(PrayError::Usage(
            "completion requires bash, zsh, or fish\nSee 'pray --help'.".to_string(),
        )),
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
