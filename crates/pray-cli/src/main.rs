mod apply_report;
mod auth_client;
mod auth_session_store;
mod cli_parse;
mod cli_parse_auth;
mod cli_parse_packages;
mod cli_parse_remote;
mod cli_release;
mod command;
mod commands_init;
mod commands_inspect;
mod commands_manifest_edit;
mod commands_materialize;
mod commands_package;
mod commands_search;
mod commands_update;
mod commands_verify;
mod completion;
mod confess;
mod help;
mod help_text;
mod invocation;
mod lockfile_ops;
mod materialize;
mod project_paths;
mod publish;
mod registry_ops;
mod revision;
mod revision_backend;
#[cfg(feature = "auth")]
mod server;
#[cfg(feature = "auth")]
mod server_auth;
#[cfg(feature = "auth")]
mod server_auth_delivery;
#[cfg(feature = "auth")]
mod server_federation;
#[cfg(feature = "auth")]
mod server_html;
#[cfg(feature = "auth")]
mod server_http;
#[cfg(feature = "auth")]
mod server_listen;
#[cfg(feature = "auth")]
mod server_registry;
#[cfg(feature = "auth")]
mod server_rpc;
#[cfg(feature = "auth")]
mod server_static;
#[cfg(feature = "auth")]
mod server_stdio;
mod sync_command;
mod sync_peers;
#[cfg(feature = "auth")]
mod token_command;
mod transport_metadata;
mod trust_command;
mod update_report;
mod update_summary;
mod yank;

pub(crate) use command::Command;
pub(crate) use project_paths::{locked_package, lockfile_path, manifest_path, resolve_project};
pub(crate) use registry_ops::{
    current_signer, current_signer_fingerprint, current_timestamp, load_registry_index,
    load_registry_package_metadata, registry_artifact_path, registry_metadata_path,
    torrent_manifest_bytes, torrent_manifest_path, write_output_bytes, write_registry_index,
    write_registry_package_metadata, write_torrent_manifest,
};

use cli_parse::parse_command;
use commands_init::{init_command, prayer_init_command, repo_init_command};
use commands_inspect::{
    clean_command, explain_command, list_command, manifest_command, outdated_command, plan_command,
    tree_command,
};
use commands_manifest_edit::{add_command, remove_command};
use commands_materialize::{apply_command, install_command};
use commands_package::{format_command, login_command, package_command, vendor_command};
use commands_search::search_command;
use commands_update::{unlock_command, update_command};
use commands_verify::{drift_command, render_command, verify_command};
use completion::completion_command;
use confess::confess_command;
use help::maybe_print_help;
use pray_core::client_trust::prepare_ephemeral_home;
use pray_core::resolve_context::ResolveOptions;
use pray_core::{PrayError, PrayResult};
use publish::publish_command;
use std::env;
use std::fs;
use std::path::PathBuf;
use sync_command::sync_command;
#[cfg(feature = "auth")]
use token_command::run_token_command;
use yank::yank_command;

fn main() {
    let code = match run(env::args().skip(1).collect()) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("{error}");
            error.exit_code()
        }
    };
    std::process::exit(code);
}

fn run(arguments: Vec<String>) -> PrayResult<()> {
    let ephemeral = arguments.iter().any(|argument| argument == "--rm");
    let trust_import = arguments.iter().any(|argument| argument == "--trust");
    let trust_global = arguments.iter().any(|argument| argument == "--global");
    let no_input = arguments.iter().any(|argument| argument == "--no-input");
    let filtered: Vec<String> = arguments
        .into_iter()
        .filter(|argument| {
            argument != "--rm"
                && argument != "--trust"
                && argument != "--global"
                && argument != "--no-input"
        })
        .collect();
    let filtered = invocation::initialize(filtered)?;
    if no_input {
        std::env::set_var("PRAY_NO_INPUT", "1");
    }
    if let Some(()) = maybe_print_help(&filtered)? {
        return Ok(());
    }
    let ephemeral_home = if ephemeral {
        Some(prepare_ephemeral_home()?)
    } else {
        None
    };
    if trust_import {
        std::env::set_var("PRAY_TRUST_IMPORT", "1");
    }
    if trust_global && !trust_import {
        return Err(PrayError::Unsupported(
            "--global requires --trust on install, update, or refresh".into(),
        ));
    }

    let result = match parse_command(filtered.clone())? {
        Command::Manifest => manifest_command(),
        Command::Init { targets } => init_command(targets),
        Command::PrayerInit => prayer_init_command(),
        Command::RepoInit => repo_init_command(),
        Command::Add {
            name,
            constraint,
            path,
        } => add_command(name, constraint, path),
        Command::Remove { name } => remove_command(name),
        Command::Update {
            package,
            major,
            latest,
            dry_run,
            json,
        } => update_command(package, major, latest, dry_run, json),
        Command::Unlock { package } => unlock_command(package),
        Command::Install {
            locked,
            frozen,
            offline,
            strict,
        } => {
            install_command(
                locked,
                frozen,
                ResolveOptions {
                    offline,
                    fail_on_yanked: strict,
                    ..ResolveOptions::default()
                },
                false,
            )?;
            Ok(())
        }
        Command::Plan { remote } => plan_command(remote),
        Command::Apply => apply_command(),
        Command::Render { check } => render_command(check),
        Command::Verify { strict } => verify_command(strict),
        Command::Drift { semantic } => drift_command(semantic),
        Command::Format => format_command(),
        Command::Package => package_command(),
        Command::Publish {
            roots,
            servers,
            signing_key,
        } => publish_command(roots, servers, signing_key),
        Command::Yank {
            package,
            version,
            root,
            undo,
        } => yank_command(package, version, root, undo),
        #[cfg(feature = "auth")]
        Command::Token { arguments } => run_token_command(arguments),
        Command::Search {
            query,
            source,
            root,
            url,
        } => search_command(query, source, root, url),
        Command::Login {
            servers,
            email,
            credential_id,
            passkey_key,
            public_key,
            ssh_agent,
        } => login_command(
            servers,
            email,
            credential_id,
            passkey_key,
            public_key,
            ssh_agent,
        ),
        #[cfg(feature = "auth")]
        Command::Serve {
            root,
            host,
            port,
            stdio,
            allow_open_push,
        } => serve_command(root, host, port, stdio, allow_open_push),
        Command::Confess {
            package,
            from_lock,
            version,
            accepted,
            rejected,
            note,
            url,
        } => confess_command(package, from_lock, version, accepted, rejected, note, url),
        Command::List => list_command(),
        Command::Outdated { remote } => outdated_command(remote),
        Command::Explain { package } => explain_command(package),
        Command::Vendor => vendor_command(),
        Command::Clean => clean_command(),
        Command::Tree => tree_command(),
        Command::Sync { root, peers } => sync_command(root, peers),
        Command::Trust { arguments } => trust_command::run_trust_command(arguments),
        Command::Upgrade => cli_release::upgrade_command(),
        Command::Version => version_command(),
        Command::Completion { shell } => completion_command(&shell),
    };

    if result.is_ok() {
        cli_release::maybe_print_upgrade_notice(&filtered);
    }

    if let Some(home) = ephemeral_home {
        let _ = fs::remove_dir_all(home);
    }
    result
}

fn version_command() -> PrayResult<()> {
    println!("pray {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

#[cfg(feature = "auth")]
fn serve_command(
    root: PathBuf,
    host: String,
    port: u16,
    stdio: bool,
    allow_open_push: bool,
) -> PrayResult<()> {
    if stdio {
        server_stdio::run_stdio_server(root)
    } else {
        server_listen::run_server(root, host, port, allow_open_push)
    }
}

#[cfg(all(test, not(feature = "auth")))]
#[path = "slim_build_tests.rs"]
mod slim_build_tests;
