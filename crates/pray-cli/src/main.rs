mod apply_report;
mod auth_client;
mod cli_parse;
mod cli_release;
mod confess;
mod help;
mod invocation;
mod materialize;
mod publish;
mod revision;
mod revision_backend;
#[cfg(feature = "auth")]
mod server;
#[cfg(feature = "auth")]
mod server_auth;
#[cfg(feature = "auth")]
mod server_federation;
#[cfg(feature = "auth")]
mod server_html;
#[cfg(feature = "auth")]
mod server_http;
#[cfg(feature = "auth")]
mod server_listen;
#[cfg(feature = "auth")]
mod server_stdio;
mod sync_command;
mod sync_peers;
mod transport_metadata;
mod trust_command;

use apply_report::{
    build_materialization_preview, materialization_preview_to_json, print_materialization_report,
    MaterializationMode, MaterializationPreview,
};
use auth_client::{
    current_signer as current_signer_from_session,
    current_signer_fingerprint as current_signer_fingerprint_from_session, login_with_passkey,
    login_with_ssh_agent,
};
use cli_parse::parse_command;
use confess::confess_command;
use materialize::{materialize_package_directory, remove_path_if_exists, write_package_archive};
#[cfg(feature = "auth")]
use pray_core::auth::RegistryAuthStore;
use pray_core::cli_suggest::unknown_command_message;
use pray_core::client_trust::prepare_ephemeral_home;
use pray_core::constraint::{latest_constraint_for_package, version_satisfies};
use pray_core::hashing::normalize_line_endings;
use pray_core::lockfile::{
    lockfiles_equivalent, read_lockfile, write_lockfile, write_lockfile_if_changed, LockedPackage,
    Lockfile,
};
use pray_core::manifest::{parse_manifest, read_manifest_text, replace_package_declaration};
use pray_core::registry::{version_is_greater_than, RegistryIndex, RegistryPackageMetadata};
use pray_core::render::{render_project, write_rendered_targets_with_previous_lockfile};
use pray_core::resolve::ResolvedProject;
use pray_core::resolve_context::ResolveOptions;
use pray_core::ssh_identity::active_ssh_user_fingerprint;
use pray_core::trust::{write_registry_trust_settings, RegistryTrustSettings};
use pray_core::verify::{drift_project, format_verification_report, verify_project};
use pray_core::{PrayError, PrayResult};
use pray_transport::{TorrentConfig, TorrentTransport};
use publish::publish_command;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use sync_command::sync_command;

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
        } => {
            install_command(
                locked,
                frozen,
                ResolveOptions {
                    offline,
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
    };

    if result.is_ok() {
        cli_release::maybe_print_upgrade_notice(&filtered);
    }

    if let Some(home) = ephemeral_home {
        let _ = fs::remove_dir_all(home);
    }
    result
}

fn maybe_print_help(arguments: &[String]) -> PrayResult<Option<()>> {
    if arguments.is_empty() {
        help::print_concise_help();
        return Ok(Some(()));
    }

    if arguments.len() == 1 && matches!(arguments[0].as_str(), "help" | "-h" | "--help") {
        help::print_concise_help();
        return Ok(Some(()));
    }

    if arguments[0] == "help" {
        let target = arguments.get(1).map(String::as_str).unwrap_or("");
        if matches!(target, "" | "-h" | "--help") {
            help::print_concise_help();
            return Ok(Some(()));
        }
        if help::print_command_help(target) {
            return Ok(Some(()));
        }
        return Err(PrayError::Usage(unknown_command_message(target)));
    }

    if let Some(position) = arguments
        .iter()
        .position(|argument| argument == "--help" || argument == "-h")
    {
        if position == 0 {
            help::print_concise_help();
            return Ok(Some(()));
        }
        let command = &arguments[0];
        if help::print_command_help(command) {
            return Ok(Some(()));
        }
        return Err(PrayError::Usage(format!("unknown command: {command}")));
    }

    Ok(None)
}

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

fn version_command() -> PrayResult<()> {
    println!("pray {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

fn manifest_command() -> PrayResult<()> {
    let manifest = load_manifest()?;
    let json = serde_json::to_string_pretty(&manifest.canonicalized())
        .map_err(|error| PrayError::Manifest(error.to_string()))?;
    println!("{json}");
    Ok(())
}

fn init_command(targets: Vec<String>) -> PrayResult<()> {
    let manifest_path = manifest_path();
    if manifest_path.exists() {
        return Err(PrayError::Manifest("Prayfile already exists".to_string()));
    }
    let mut text = String::new();
    text.push_str("prayfile \"1\"\n");
    for target in if targets.is_empty() {
        vec!["tool_a".to_string()]
    } else {
        targets
    } {
        text.push_str(&format!(
            "target :{} do\n  output \"{}.md\"\nend\n",
            target,
            default_output_for_target(&target)
        ));
    }
    fs::write(manifest_path, text)?;
    Ok(())
}

fn prayer_init_command() -> PrayResult<()> {
    let root = env::current_dir()?;
    let package_name = root
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("prayer-package")
        .to_string();
    let prayspec_path = root.join(format!("{package_name}.prayspec"));
    if prayspec_path.exists() {
        return Err(PrayError::Manifest(format!(
            "package spec already exists: {}",
            prayspec_path.display()
        )));
    }

    fs::write(
        &prayspec_path,
        format!(
            r#"Package::Specification.new do |spec|
  spec.name = "{package_name}"
  spec.version = "0.1.0"
  spec.summary = "Describe this package"
  spec.files = ["README.md"]
  spec.exports = {{}}
end
"#
        ),
    )?;
    if !root.join("README.md").exists() {
        fs::write(root.join("README.md"), format!("# {package_name}\n"))?;
    }
    fs::create_dir_all(root.join("exports"))?;
    Ok(())
}

fn repo_init_command() -> PrayResult<()> {
    let root = env::current_dir()?;
    let distribution_root = repo_distribution_root(&root);
    let index_path = distribution_root.join("v1/index.json");
    let trust_path = distribution_root.join("v1/trust.json");
    if index_path.exists() || trust_path.exists() {
        return Err(PrayError::Manifest(
            "distribution repo already exists".to_string(),
        ));
    }

    fs::create_dir_all(distribution_root.join("v1/packages"))?;
    fs::create_dir_all(distribution_root.join("v1/artifacts"))?;
    write_registry_index(
        &distribution_root,
        &RegistryIndex {
            spec: "prayfile-distribution-1".to_string(),
            packages: Vec::new(),
        },
    )?;
    write_registry_trust_settings(&distribution_root, &RegistryTrustSettings::default())?;
    Ok(())
}

fn repo_distribution_root(root: &Path) -> PathBuf {
    if root.file_name().and_then(|value| value.to_str()) == Some("prayers") {
        root.to_path_buf()
    } else {
        root.join("prayers")
    }
}

fn add_command(name: String, constraint: Option<String>, path: Option<String>) -> PrayResult<()> {
    let manifest_path = manifest_path();
    let manifest_text = read_manifest_text(&manifest_path)?;
    let manifest = parse_manifest(&manifest_text)?;
    if manifest.packages.iter().any(|package| package.name == name) {
        return Err(PrayError::Manifest(format!(
            "package {name} already exists"
        )));
    }

    let declaration = if let Some(path) = path {
        if let Some(constraint) = constraint {
            format!("agent \"{name}\", \"{constraint}\", path: \"{path}\"")
        } else {
            format!("agent \"{name}\", path: \"{path}\"")
        }
    } else if let Some(constraint) = constraint {
        format!("agent \"{name}\", \"{constraint}\"")
    } else {
        format!("agent \"{name}\"")
    };

    fs::write(
        manifest_path,
        insert_manifest_statement(&manifest_text, &declaration),
    )?;
    Ok(())
}

fn remove_command(name: String) -> PrayResult<()> {
    let manifest_path = manifest_path();
    let manifest_text = read_manifest_text(&manifest_path)?;
    let manifest = parse_manifest(&manifest_text)?;
    if !manifest.packages.iter().any(|package| package.name == name) {
        return Err(PrayError::Manifest(format!("package {name} not found")));
    }

    fs::write(
        manifest_path,
        remove_manifest_statement(&manifest_text, &name),
    )?;
    install_command(false, false, ResolveOptions::default(), false)?;
    Ok(())
}

fn update_command(
    package: Option<String>,
    major: bool,
    latest: bool,
    dry_run: bool,
    json: bool,
) -> PrayResult<()> {
    if major && latest {
        return Err(PrayError::Unsupported(
            "use either --major or --latest, not both".to_string(),
        ));
    }
    if major {
        if package.is_none() {
            return Err(PrayError::Unsupported(
                "major updates require a package name".to_string(),
            ));
        }
        if dry_run {
            return Err(PrayError::Unsupported(
                "major updates are not supported with --dry-run".to_string(),
            ));
        }
        return update_latest_command(package, json);
    }
    if latest {
        if dry_run {
            return Err(PrayError::Unsupported(
                "--latest is not supported with --dry-run".to_string(),
            ));
        }
        return update_latest_command(package, json);
    }

    if dry_run {
        return preview_remote_updates(package.as_deref(), json);
    }

    update_command_with_manifest_constraints(package, json, Vec::new())
}

fn update_latest_command(package: Option<String>, json: bool) -> PrayResult<()> {
    let manifest_path = manifest_path();
    let manifest_text = read_manifest_text(&manifest_path)?;
    let preview_options = constraint_preview_options();
    let project = resolve_project_with_options(&manifest_path, &preview_options)?;

    if let Some(package_name) = &package {
        if !project
            .manifest
            .packages
            .iter()
            .any(|declaration| declaration.name == *package_name)
        {
            return Err(PrayError::Manifest(format!(
                "package {package_name} not found"
            )));
        }
    }

    let mut updated_text = manifest_text;
    let mut manifest_updates = Vec::new();

    for resolved in &project.packages {
        if let Some(package_name) = &package {
            if resolved.declaration.name != *package_name {
                continue;
            }
        }
        let Some(registry_latest_version) = &resolved.registry_latest_version else {
            continue;
        };
        if version_satisfies(registry_latest_version, &resolved.declaration.constraint)? {
            continue;
        }
        let new_constraint = latest_constraint_for_package(
            &resolved.declaration.constraint,
            registry_latest_version,
        )?;
        if !version_satisfies(registry_latest_version, &new_constraint)? {
            return Err(PrayError::Resolution(format!(
                "derived constraint {new_constraint} does not admit registry latest {registry_latest_version} for {}",
                resolved.declaration.name
            )));
        }
        let mut updated_declaration = resolved.declaration.clone();
        let previous_constraint = updated_declaration.constraint.clone();
        updated_declaration.constraint = new_constraint.clone();
        updated_text = replace_package_declaration(&updated_text, &updated_declaration)?;
        manifest_updates.push((
            resolved.declaration.name.clone(),
            previous_constraint,
            new_constraint,
            registry_latest_version.clone(),
        ));
    }

    let manifest_constraint_updates: Vec<serde_json::Value> = manifest_updates
        .iter()
        .map(
            |(name, previous_constraint, new_constraint, registry_latest_version)| {
                serde_json::json!({
                    "name": name,
                    "from_constraint": previous_constraint,
                    "to_constraint": new_constraint,
                    "registry_latest_version": registry_latest_version,
                })
            },
        )
        .collect();

    if manifest_updates.is_empty() {
        if json {
            let current_lockfile = read_lockfile(&lockfile_path()).unwrap_or_default();
            print_update_json_report(
                &manifest_constraint_updates,
                None,
                None,
                &current_lockfile,
                package.as_deref(),
                &project,
            )?;
            return Ok(());
        }
        println!("All package constraints already allow registry latest versions");
    } else {
        if !json {
            for (name, previous_constraint, new_constraint, registry_latest_version) in
                &manifest_updates
            {
                println!(
                    "Prayfile: {name} constraint {previous_constraint} -> {new_constraint} (registry latest {registry_latest_version})"
                );
            }
        }
        fs::write(&manifest_path, updated_text)?;
    }

    update_command_with_manifest_constraints(package, json, manifest_constraint_updates)
}

fn update_command_with_manifest_constraints(
    package: Option<String>,
    json: bool,
    manifest_constraint_updates: Vec<serde_json::Value>,
) -> PrayResult<()> {
    let manifest_path = manifest_path();
    let manifest_text = read_manifest_text(&manifest_path)?;
    if let Some(package_name) = &package {
        let manifest = parse_manifest(&manifest_text)?;
        if !manifest
            .packages
            .iter()
            .any(|declaration| declaration.name == *package_name)
        {
            return Err(PrayError::Manifest(format!(
                "package {package_name} not found"
            )));
        }
    }

    let previous_lockfile = read_lockfile(&lockfile_path()).ok();
    let mut resolve_options = ResolveOptions {
        refresh_source_revisions: true,
        ..ResolveOptions::default()
    };
    if let Some(package_name) = &package {
        resolve_options
            .unlocked_packages
            .insert(package_name.clone());
    } else {
        resolve_options.ignore_locked_versions = true;
    }
    let install_preview = install_command(false, false, resolve_options.clone(), json)?;
    let updated_lockfile = read_lockfile(&lockfile_path())?;
    let refreshed_project = resolve_project_with_options(&manifest_path, &resolve_options)?;
    let merged_lockfile = if let (Some(previous_lockfile), Some(package_name)) =
        (previous_lockfile.as_ref(), package.as_deref())
    {
        merge_selected_package_update(previous_lockfile, &updated_lockfile, package_name)
    } else {
        updated_lockfile
    };
    if package.is_some() {
        write_lockfile(&lockfile_path(), &merged_lockfile)?;
    }
    if json {
        print_update_json_report(
            &manifest_constraint_updates,
            install_preview.as_ref(),
            previous_lockfile.as_ref(),
            &merged_lockfile,
            package.as_deref(),
            &refreshed_project,
        )?;
        return Ok(());
    }
    let update_reported = print_update_summary(
        previous_lockfile.as_ref(),
        &merged_lockfile,
        package.as_deref(),
        &refreshed_project,
        "Update summary",
    )?;
    let _ =
        print_constraint_blocked_packages(&refreshed_project, "Update summary", !update_reported)?;
    Ok(())
}

fn unlock_command(package: String) -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    if !project
        .manifest
        .packages
        .iter()
        .any(|declaration| declaration.name == package)
    {
        return Err(PrayError::Manifest(format!("package {package} not found")));
    }
    let previous_lockfile = read_lockfile(&lockfile_path())?;
    let mut options = ResolveOptions::default();
    options.unlocked_packages.insert(package.clone());
    let project = resolve_project_with_options(&manifest_path(), &options)?;
    let rendered = render_project(&project)?;
    let updated_lockfile = build_lockfile(&project, &rendered)?;
    let merged_lockfile =
        merge_selected_package_update(&previous_lockfile, &updated_lockfile, &package);
    write_lockfile(&lockfile_path(), &merged_lockfile)?;
    write_rendered_targets_with_previous_lockfile(&project, &rendered, Some(&previous_lockfile))?;
    println!("Unlocked {package}");
    Ok(())
}

fn insert_manifest_statement(text: &str, statement: &str) -> String {
    let mut lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
    let insertion_index = lines
        .iter()
        .position(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("local ") || trimmed.starts_with("render ")
        })
        .unwrap_or(lines.len());
    lines.insert(insertion_index, statement.to_string());
    let mut output = lines.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

fn remove_manifest_statement(text: &str, name: &str) -> String {
    let mut lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
    let package_prefix = format!("agent \"{name}\"");
    if let Some(index) = lines.iter().position(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with(&package_prefix) || trimmed.starts_with(&format!("agent '{name}'"))
    }) {
        lines.remove(index);
        if index < lines.len() && lines[index].trim().is_empty() {
            lines.remove(index);
        } else if index > 0 && lines[index - 1].trim().is_empty() {
            lines.remove(index - 1);
        }
    }
    let mut output = lines.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

fn constraint_preview_options() -> ResolveOptions {
    ResolveOptions {
        refresh_source_revisions: true,
        ignore_locked_versions: true,
        ..ResolveOptions::default()
    }
}

fn remote_preview_options() -> ResolveOptions {
    ResolveOptions {
        refresh_source_revisions: true,
        ignore_locked_versions: true,
        ..ResolveOptions::default()
    }
}

fn preview_remote_updates(selected_package: Option<&str>, json: bool) -> PrayResult<()> {
    if json {
        return Err(PrayError::Unsupported(
            "--json is not supported with --dry-run".to_string(),
        ));
    }
    let previous_lockfile = read_lockfile(&lockfile_path()).ok();
    let project = resolve_project_with_options(&manifest_path(), &remote_preview_options())?;
    let rendered = render_project(&project)?;
    let updated_lockfile = build_lockfile(&project, &rendered)?;
    if print_update_summary(
        previous_lockfile.as_ref(),
        &updated_lockfile,
        selected_package,
        &project,
        "Remote update preview",
    )? {
        print_constraint_blocked_packages(&project, "Remote update preview", false)?;
        return Ok(());
    }
    if print_constraint_blocked_packages(&project, "Outdated packages", true)? {
        return Ok(());
    }
    println!("Outdated packages");
    println!("All packages up to date");
    Ok(())
}

#[cfg(feature = "auth")]
fn install_command(
    locked: bool,
    frozen: bool,
    resolve_options: ResolveOptions,
    silent_report: bool,
) -> PrayResult<Option<MaterializationPreview>> {
    let report_mode = if locked {
        None
    } else {
        Some(MaterializationMode::Install)
    };
    materialize_command(locked, frozen, report_mode, resolve_options, silent_report)
}

fn apply_command() -> PrayResult<()> {
    materialize_command(
        false,
        false,
        Some(MaterializationMode::Apply),
        ResolveOptions::default(),
        false,
    )?;
    Ok(())
}

fn resolve_project_for_materialization(
    resolve_options: &ResolveOptions,
    locked: bool,
    frozen: bool,
) -> PrayResult<ResolvedProject> {
    resolve_project_with_git_refresh_fallback(&manifest_path(), resolve_options, !locked && !frozen)
}

fn materialize_command(
    locked: bool,
    frozen: bool,
    report_mode: Option<MaterializationMode>,
    resolve_options: ResolveOptions,
    silent_report: bool,
) -> PrayResult<Option<MaterializationPreview>> {
    let project = resolve_project_for_materialization(&resolve_options, locked, frozen)?;
    let rendered = render_project(&project)?;
    let lockfile_path = lockfile_path();
    if locked {
        let lockfile = ensure_existing_lockfile(&lockfile_path)?;
        ensure_lockfile_current(&project, &rendered, &lockfile)?;
        if frozen {
            ensure_rendered_outputs_current(&project, &rendered)?;
            return Ok(None);
        }
        write_rendered_targets_with_previous_lockfile(&project, &rendered, Some(&lockfile))?;
        return Ok(None);
    }

    let lockfile = build_lockfile(&project, &rendered)?;
    let previous_lockfile = read_lockfile(&lockfile_path).ok();
    let preview = if report_mode.is_some() {
        Some(build_materialization_preview(
            &project,
            &rendered,
            &lockfile,
            &lockfile_path,
            previous_lockfile.as_ref(),
        )?)
    } else {
        None
    };
    write_lockfile_if_changed(&lockfile_path, &lockfile)?;
    write_rendered_targets_with_previous_lockfile(&project, &rendered, previous_lockfile.as_ref())?;
    if !silent_report {
        if let (Some(preview), Some(mode)) = (&preview, report_mode) {
            print_materialization_report(preview, mode);
        }
    }
    Ok(preview)
}

fn plan_command(remote: bool) -> PrayResult<()> {
    let options = if remote {
        remote_preview_options()
    } else {
        ResolveOptions::default()
    };
    let project = if remote {
        resolve_project_with_options(&manifest_path(), &options)?
    } else {
        resolve_project_for_materialization(&options, false, false)?
    };
    let rendered = render_project(&project)?;
    let lockfile = build_lockfile(&project, &rendered)?;
    let previous_lockfile = read_lockfile(&lockfile_path()).ok();
    let preview = build_materialization_preview(
        &project,
        &rendered,
        &lockfile,
        &lockfile_path(),
        previous_lockfile.as_ref(),
    )?;
    print_materialization_report(&preview, MaterializationMode::Plan);
    Ok(())
}

fn clean_command() -> PrayResult<()> {
    remove_path_if_exists(Path::new(".pray/cache"))?;
    remove_path_if_exists(Path::new(".pray/vendor"))?;
    remove_path_if_exists(Path::new(".pray/state.json"))?;
    Ok(())
}

fn tree_command() -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    let package_map: std::collections::BTreeMap<String, &pray_core::resolve::ResolvedPackage> =
        project
            .packages
            .iter()
            .map(|package| (package.declaration.name.clone(), package))
            .collect();
    let mut lines = vec!["Dependency tree".to_string()];
    for package in &project.packages {
        let mut ancestry = std::collections::BTreeSet::new();
        render_tree_node(package, &package_map, 0, &mut ancestry, &mut lines);
    }
    println!("{}", lines.join("\n"));
    Ok(())
}

fn list_command() -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    let mut lines = vec!["Package list".to_string()];
    for package in &project.packages {
        lines.push(format!(
            "{} {} source={} exports={}",
            package.declaration.name,
            package.spec.version,
            package_source_summary(package),
            format_list(&package.selected_exports)
        ));
    }
    println!("{}", lines.join("\n"));
    Ok(())
}

fn outdated_command(remote: bool) -> PrayResult<()> {
    if remote {
        return preview_remote_updates(None, false);
    }

    let previous_lockfile = read_lockfile(&lockfile_path()).ok();
    let project = resolve_project_with_options(&manifest_path(), &constraint_preview_options())?;
    let rendered = render_project(&project)?;
    let latest_lockfile = build_lockfile(&project, &rendered)?;
    let mut reported = print_update_summary(
        previous_lockfile.as_ref(),
        &latest_lockfile,
        None,
        &project,
        "Outdated packages",
    )?;
    reported |= print_constraint_blocked_packages(&project, "Outdated packages", !reported)?;
    if !reported {
        println!("Outdated packages");
        println!("All packages up to date");
    }
    Ok(())
}

fn explain_command(package_name: String) -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    let lockfile = read_lockfile(&lockfile_path()).ok();
    let package = project
        .packages
        .iter()
        .find(|package| package.declaration.name == package_name)
        .ok_or_else(|| PrayError::Resolution(format!("package {package_name} not found")))?;
    let lockfile_package = lockfile
        .as_ref()
        .and_then(|lockfile| locked_package(lockfile, package));

    let mut lines = vec!["Package explanation".to_string()];
    lines.push(format!("name: {}", package.declaration.name));
    lines.push(format!("constraint: {}", package.declaration.constraint));
    lines.push(format!("resolved version: {}", package.spec.version));
    if let Some(registry_latest_version) = &package.registry_latest_version {
        lines.push(format!("registry latest: {registry_latest_version}"));
        if version_is_greater_than(registry_latest_version, &package.spec.version)? {
            lines.push(format!(
                "constraint blocks upgrade: {} allows up to {}, registry has {registry_latest_version}",
                package.declaration.constraint, package.spec.version
            ));
        }
    }
    lines.push(format!("source: {}", package_source_summary(package)));
    lines.push(format!(
        "exports: {}",
        format_list(&package.selected_exports)
    ));
    lines.push(format!(
        "dependencies: {}",
        format_list(
            &package
                .spec
                .dependencies
                .iter()
                .map(|dependency| dependency.name.clone())
                .collect::<Vec<_>>()
        )
    ));
    lines.push(format!("tree hash: {}", package.tree_hash));
    lines.push(format!("artifact hash: {}", package.artifact_hash));

    match lockfile_package {
        Some(record) => {
            lines.push(format!("lockfile version: {}", record.version));
            lines.push(format!("lockfile path: {}", record.path));
            lines.push(format!(
                "lockfile exports: {}",
                format_list(&record.exports)
            ));
        }
        None => lines.push("lockfile record: missing".to_string()),
    }

    println!("{}", lines.join("\n"));
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

pub(crate) fn current_signer() -> PrayResult<String> {
    let session_root = workspace_root();
    if let Some(email) = current_signer_from_session(&session_root) {
        return Ok(email);
    }

    if let Ok(token) = std::env::var("PRAY_SESSION_TOKEN") {
        #[cfg(feature = "auth")]
        {
            let auth_root = std::env::var("PRAY_AUTH_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|_| session_root.clone());
            if let Ok(store) = RegistryAuthStore::open(&auth_root) {
                if let Ok(Some(session)) = store.resolve_session(&token) {
                    return Ok(session.email);
                }
            }
        }

        #[cfg(not(feature = "auth"))]
        {
            let _ = token;
            return Err(PrayError::Unsupported(
                "this build was compiled without auth support".to_string(),
            ));
        }
    }

    Ok(std::env::var("PRAY_SIGNER")
        .or_else(|_| std::env::var("USER"))
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string()))
}

pub(crate) fn current_signer_fingerprint() -> Option<String> {
    if let Some(fingerprint) = active_ssh_user_fingerprint() {
        return Some(fingerprint);
    }
    current_signer_fingerprint_from_session(&workspace_root())
}

#[cfg(all(test, not(feature = "auth")))]
mod slim_build_tests {
    use super::*;

    #[test]
    fn omits_the_serve_command() {
        assert!(matches!(
            parse_command(vec!["serve".to_string()]),
            Err(PrayError::Usage(message)) if message == "unknown command: serve"
        ));
    }

    #[test]
    fn rejects_session_tokens_without_auth_storage() {
        std::env::set_var("PRAY_SESSION_TOKEN", "session-token");

        let error = current_signer().expect_err("session tokens require auth storage");

        std::env::remove_var("PRAY_SESSION_TOKEN");
        assert!(matches!(
            error,
            PrayError::Unsupported(message) if message == "this build was compiled without auth support"
        ));
    }
}

pub(crate) fn current_timestamp() -> PrayResult<String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| PrayError::Resolution(error.to_string()))
        .map(|duration| duration.as_secs().to_string())
}

fn format_command() -> PrayResult<()> {
    let path = manifest_path();
    let original = read_manifest_text(&path)?;
    let manifest = parse_manifest(&original)?;
    let project = resolve_project_with_options(
        &path,
        &ResolveOptions {
            offline: true,
            ..ResolveOptions::default()
        },
    )
    .or_else(|_| resolve_project(&path))?;
    let hints = pray_core::format_manifest::classify_format_hints(&project);
    let formatted = pray_core::format_manifest::format_recommended(&manifest, &hints)?;
    if formatted != original {
        fs::write(&path, formatted)?;
    }

    if let Ok(lockfile) = read_lockfile(&lockfile_path()) {
        for target in &lockfile.target {
            for output in &target.outputs {
                let output_path = Path::new(output);
                if !output_path.exists() {
                    continue;
                }
                let original_output = fs::read_to_string(output_path)?;
                let formatted_output =
                    format_marker_comments(&normalize_line_endings(&original_output));
                if formatted_output != original_output {
                    fs::write(output_path, formatted_output)?;
                }
            }
        }
    }
    Ok(())
}

fn package_command() -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    for package in &project.packages {
        let output_path = package_archive_path(&package.declaration.name, &package.spec.version);
        write_package_archive(package, &output_path)?;
    }
    Ok(())
}

fn login_command(
    servers: Vec<String>,
    email: String,
    credential_id: Option<String>,
    passkey_key: Option<PathBuf>,
    public_key: Option<PathBuf>,
    ssh_agent: bool,
) -> PrayResult<()> {
    let session_root = workspace_root();
    for server in servers {
        let session = if let Some(passkey_key) = &passkey_key {
            let credential_id = credential_id.as_ref().ok_or_else(|| {
                PrayError::Unsupported("passkey login requires --credential-id".to_string())
            })?;
            login_with_passkey(&server, credential_id, passkey_key, &session_root)?
        } else if ssh_agent {
            let public_key = public_key.as_ref().ok_or_else(|| {
                PrayError::Unsupported("ssh-agent login requires --public-key".to_string())
            })?;
            login_with_ssh_agent(&server, public_key, &session_root)?
        } else {
            return Err(PrayError::Unsupported(
                "login requires an authentication mode".to_string(),
            ));
        };
        if session.email != email {
            return Err(PrayError::Resolution(format!(
                "login completed for {} but {} was requested",
                session.email, email
            )));
        }
        println!(
            "logged in as {} via {} on {}",
            session.email, session.kind, server
        );
    }
    Ok(())
}

fn vendor_command() -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    for package in &project.packages {
        let output_directory =
            vendor_package_path(&package.declaration.name, &package.spec.version);
        materialize_package_directory(package, &output_directory)?;
    }
    Ok(())
}

pub(crate) fn load_registry_index(root: &Path) -> PrayResult<RegistryIndex> {
    let path = root.join("v1/index.json");
    let Ok(text) = fs::read_to_string(&path) else {
        return Ok(RegistryIndex {
            spec: "prayfile-distribution-1".to_string(),
            packages: Vec::new(),
        });
    };
    let index: RegistryIndex = serde_json::from_str(&text).map_err(|error| PrayError::Parse {
        kind: "registry index",
        message: error.to_string(),
    })?;
    if index.spec != "prayfile-distribution-1" {
        return Err(PrayError::Resolution(format!(
            "unsupported registry index spec: {}",
            index.spec
        )));
    }
    Ok(index)
}

pub(crate) fn load_registry_package_metadata(
    path: &Path,
    package_name: &str,
) -> PrayResult<RegistryPackageMetadata> {
    if path.exists() {
        let text = fs::read_to_string(path)?;
        let metadata: RegistryPackageMetadata =
            serde_json::from_str(&text).map_err(|error| PrayError::Parse {
                kind: "registry metadata",
                message: error.to_string(),
            })?;
        if metadata.name != package_name {
            return Err(PrayError::Resolution(format!(
                "registry metadata name mismatch: expected {}, found {}",
                package_name, metadata.name
            )));
        }
        Ok(metadata)
    } else {
        Ok(RegistryPackageMetadata {
            name: package_name.to_string(),
            versions: Vec::new(),
        })
    }
}

pub(crate) fn write_registry_index(root: &Path, index: &RegistryIndex) -> PrayResult<()> {
    let path = root.join("v1/index.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(index)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
    )?;
    Ok(())
}

pub(crate) fn write_registry_package_metadata(
    path: &Path,
    metadata: &RegistryPackageMetadata,
) -> PrayResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(metadata)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
    )?;
    Ok(())
}

pub(crate) fn registry_metadata_path(root: &Path, package_name: &str) -> PathBuf {
    root.join("v1/packages")
        .join(package_name)
        .with_extension("json")
}

pub(crate) fn registry_artifact_path(package_name: &str, version: &str) -> String {
    let artifact_name = format!("{}-{}.praypkg", package_name.replace('/', "-"), version);
    format!("v1/artifacts/{package_name}/{version}/{artifact_name}")
}

pub(crate) fn torrent_manifest_path(artifact_path: &str) -> String {
    format!("{artifact_path}.praytorrent.json")
}

pub(crate) fn torrent_manifest_bytes(
    package: &pray_core::resolve::ResolvedPackage,
    artifact_path: &str,
    archive_bytes: &[u8],
) -> PrayResult<Vec<u8>> {
    let torrent_config = TorrentConfig::default();
    let manifest = TorrentTransport::build_manifest(
        package.declaration.name.clone(),
        package.spec.version.clone(),
        artifact_path.to_string(),
        archive_bytes,
        torrent_config.piece_size,
        vec![artifact_path.to_string()],
        torrent_config.bootstrap_trackers,
    );
    serde_json::to_vec_pretty(&manifest).map_err(|error| PrayError::Manifest(error.to_string()))
}

pub(crate) fn write_torrent_manifest(
    root: &Path,
    package: &pray_core::resolve::ResolvedPackage,
    artifact_path: &str,
    archive_bytes: &[u8],
) -> PrayResult<()> {
    let manifest_path = root.join(torrent_manifest_path(artifact_path));
    write_output_bytes(
        &manifest_path,
        &torrent_manifest_bytes(package, artifact_path, archive_bytes)?,
    )
}

pub(crate) fn write_output_bytes(path: &Path, bytes: &[u8]) -> PrayResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)?;
    Ok(())
}

fn render_command(check_only: bool) -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    let rendered = render_project(&project)?;
    if check_only {
        ensure_rendered_outputs_current(&project, &rendered)?;
        return Ok(());
    }
    let previous_lockfile = read_lockfile(&lockfile_path()).ok();
    let lockfile = build_lockfile(&project, &rendered)?;
    write_lockfile(&lockfile_path(), &lockfile)?;
    write_rendered_targets_with_previous_lockfile(&project, &rendered, previous_lockfile.as_ref())?;
    Ok(())
}

fn verify_command(strict: bool) -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    let lockfile = read_lockfile(&lockfile_path())?;
    let report = verify_project(&project, &lockfile, strict)?;
    if !report.is_clean() {
        eprintln!("{}", format_verification_report(&report));
    }
    Ok(())
}

fn drift_command(semantic: bool) -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    let lockfile = read_lockfile(&lockfile_path())?;
    if semantic {
        drift_semantic_command(&project, &lockfile)
    } else {
        drift_project(&project, &lockfile)?;
        Ok(())
    }
}

fn drift_semantic_command(
    project: &pray_core::resolve::ResolvedProject,
    lockfile: &Lockfile,
) -> PrayResult<()> {
    let lock_versions: std::collections::BTreeMap<&str, (&str, usize)> = lockfile
        .package
        .iter()
        .map(|package| {
            let managed_span_count = lockfile
                .managed_span
                .iter()
                .filter(|span| span.package == package.name)
                .count();
            (
                package.name.as_str(),
                (package.version.as_str(), managed_span_count),
            )
        })
        .collect();

    let mut lines = Vec::new();
    for package in &project.packages {
        let Some((locked_version, managed_span_count)) =
            lock_versions.get(package.declaration.name.as_str())
        else {
            continue;
        };
        if *locked_version != package.spec.version {
            lines.push(format!(
                "{} {} -> {} would change {} managed spans",
                package.declaration.name, locked_version, package.spec.version, managed_span_count,
            ));
        }
    }

    if lines.is_empty() {
        return Ok(());
    }

    let mut report = String::from("Semantic diff");
    for line in lines {
        report.push('\n');
        report.push_str(&line);
    }
    Err(PrayError::Verify(report))
}

fn build_lockfile(
    project: &pray_core::resolve::ResolvedProject,
    rendered: &[pray_core::render::RenderedTarget],
) -> PrayResult<Lockfile> {
    Ok(pray_core::lockfile::build_lockfile(
        project.lockfile_hash()?,
        project.environment.clone(),
        &project.project_root,
        &project.manifest.sources,
        &project.manifest.targets,
        rendered,
        &project.packages,
        &project.source_revisions,
        &project.source_host_keys,
    ))
}

pub(crate) fn manifest_path() -> PathBuf {
    invocation::manifest_path()
}

pub(crate) fn lockfile_path() -> PathBuf {
    invocation::lockfile_path()
}

fn workspace_root() -> PathBuf {
    invocation::project_root()
}

fn resolve_project_with_options(
    _manifest_path: &Path,
    options: &ResolveOptions,
) -> PrayResult<ResolvedProject> {
    invocation::resolve_current_project(options)
}

pub(crate) fn resolve_project(_manifest_path: &Path) -> PrayResult<ResolvedProject> {
    invocation::resolve_current_project(&ResolveOptions::default())
}

fn resolve_project_with_git_refresh_fallback(
    _manifest_path: &Path,
    options: &ResolveOptions,
    allow_git_refresh_fallback: bool,
) -> PrayResult<ResolvedProject> {
    match invocation::resolve_current_project(options) {
        Ok(project) => Ok(project),
        Err(PrayError::Resolution(message))
            if allow_git_refresh_fallback
                && !options.offline
                && !options.refresh_source_revisions
                && message.contains("no registry version") =>
        {
            let refreshed_options = ResolveOptions {
                refresh_source_revisions: true,
                ..options.clone()
            };
            invocation::resolve_current_project(&refreshed_options)
        }
        Err(error) => Err(error),
    }
}

fn load_manifest() -> PrayResult<pray_core::manifest::Manifest> {
    let text = read_manifest_text(&manifest_path())?;
    let manifest = parse_manifest(&text)?;
    for warning in manifest.deprecation_warnings() {
        eprintln!("{warning}");
    }
    Ok(manifest)
}

fn default_output_for_target(target: &str) -> String {
    match target {
        "tool_a" => "INSTRUCTIONS".to_string(),
        "tool_b" => "TOOL_B".to_string(),
        other => other.to_uppercase(),
    }
}

fn ensure_existing_lockfile(path: &Path) -> PrayResult<Lockfile> {
    if !path.exists() {
        return Err(PrayError::Verify(
            "missing Prayfile.lock; run pray install first".to_string(),
        ));
    }
    read_lockfile(path)
}

fn ensure_lockfile_current(
    project: &pray_core::resolve::ResolvedProject,
    rendered: &[pray_core::render::RenderedTarget],
    existing: &Lockfile,
) -> PrayResult<()> {
    let current = build_lockfile(project, rendered)?;
    if !lockfiles_equivalent(&current, existing) {
        return Err(PrayError::Verify(
            "lockfile needs update; rerun pray install to refresh Prayfile.lock".to_string(),
        ));
    }
    Ok(())
}

fn ensure_rendered_outputs_current(
    project: &pray_core::resolve::ResolvedProject,
    rendered: &[pray_core::render::RenderedTarget],
) -> PrayResult<()> {
    for target in rendered {
        let path = project.project_root.join(&target.path);
        let on_disk = fs::read_to_string(&path).map_err(PrayError::from)?;
        if on_disk != target.content {
            return Err(PrayError::Render(format!(
                "{} is stale; rerun pray install to regenerate it or pray plan to inspect the diff",
                path.display()
            )));
        }
    }
    Ok(())
}

fn package_archive_path(name: &str, version: &str) -> PathBuf {
    PathBuf::from(format!("{}-{}.praypkg", name.replace('/', "-"), version))
}

fn vendor_package_path(name: &str, version: &str) -> PathBuf {
    PathBuf::from(".pray/vendor")
        .join(name.replace('/', "-"))
        .join(version)
}

fn merge_selected_package_update(
    previous: &Lockfile,
    updated: &Lockfile,
    selected_package: &str,
) -> Lockfile {
    let mut merged = updated.clone();
    for package in &mut merged.package {
        if package.name == selected_package {
            continue;
        }
        if let Some(previous_package) = previous
            .package
            .iter()
            .find(|locked_package| locked_package.name == package.name)
        {
            package.version = previous_package.version.clone();
        }
    }
    merged
}

fn constraint_blocked_packages_json(
    project: &ResolvedProject,
) -> PrayResult<Vec<serde_json::Value>> {
    let mut packages = Vec::new();
    for package in &project.packages {
        let Some(registry_latest_version) = &package.registry_latest_version else {
            continue;
        };
        if registry_latest_version == &package.spec.version {
            continue;
        }
        if version_is_greater_than(registry_latest_version, &package.spec.version)? {
            packages.push(serde_json::json!({
                "name": package.declaration.name,
                "resolved_version": package.spec.version,
                "registry_latest_version": registry_latest_version,
                "constraint": package.declaration.constraint,
            }));
        }
    }
    packages.sort_by(|left, right| {
        left["name"]
            .as_str()
            .unwrap_or_default()
            .cmp(right["name"].as_str().unwrap_or_default())
    });
    Ok(packages)
}

fn constraint_blocked_package_lines(project: &ResolvedProject) -> PrayResult<Vec<String>> {
    let mut lines = Vec::new();
    for package in &project.packages {
        let Some(registry_latest_version) = &package.registry_latest_version else {
            continue;
        };
        if registry_latest_version == &package.spec.version {
            continue;
        }
        if version_is_greater_than(registry_latest_version, &package.spec.version)? {
            lines.push(format!(
                "Available package {} {} -> {} (blocked by {})",
                package.declaration.name,
                package.spec.version,
                registry_latest_version,
                package.declaration.constraint,
            ));
        }
    }
    lines.sort();
    Ok(lines)
}

fn print_constraint_blocked_packages(
    project: &ResolvedProject,
    title: &str,
    print_title: bool,
) -> PrayResult<bool> {
    let lines = constraint_blocked_package_lines(project)?;
    if lines.is_empty() {
        return Ok(false);
    }
    if print_title {
        println!("{title}");
    }
    for line in lines {
        println!("{line}");
    }
    Ok(true)
}

fn print_update_summary(
    previous: Option<&Lockfile>,
    updated: &Lockfile,
    selected_package: Option<&str>,
    project: &pray_core::resolve::ResolvedProject,
    title: &str,
) -> PrayResult<bool> {
    let report = build_update_summary(previous, updated, selected_package, project)?;
    if report.lines.is_empty() {
        return Ok(false);
    }

    println!("{title}");
    for line in report.lines {
        println!("{line}");
    }
    Ok(true)
}

fn print_update_json_report(
    manifest_constraint_updates: &[serde_json::Value],
    install_preview: Option<&MaterializationPreview>,
    previous: Option<&Lockfile>,
    updated: &Lockfile,
    selected_package: Option<&str>,
    project: &ResolvedProject,
) -> PrayResult<()> {
    let summary = build_update_summary(previous, updated, selected_package, project)?;
    let constraint_blocked_packages = constraint_blocked_packages_json(project)?;
    let status = if manifest_constraint_updates.is_empty()
        && summary.updated_packages.is_empty()
        && install_preview.is_none()
        && constraint_blocked_packages.is_empty()
    {
        "up_to_date"
    } else {
        "updated"
    };
    let mut output = serde_json::json!({
        "status": status,
        "manifest_constraint_updates": manifest_constraint_updates,
        "updated_packages": summary.updated_packages,
        "constraint_blocked_packages": constraint_blocked_packages,
    });
    if let Some(preview) = install_preview {
        output["install"] = materialization_preview_to_json(preview, MaterializationMode::Install);
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| PrayError::Manifest(error.to_string()))?
    );
    Ok(())
}

struct UpdateSummaryReport {
    lines: Vec<String>,
    updated_packages: Vec<serde_json::Value>,
}

fn build_update_summary(
    previous: Option<&Lockfile>,
    updated: &Lockfile,
    selected_package: Option<&str>,
    project: &pray_core::resolve::ResolvedProject,
) -> PrayResult<UpdateSummaryReport> {
    let previous_packages: std::collections::BTreeMap<&str, &LockedPackage> = previous
        .into_iter()
        .flat_map(|lockfile| lockfile.package.iter())
        .map(|package| (package.name.as_str(), package))
        .collect();
    let package_sources: std::collections::BTreeMap<&str, String> = project
        .packages
        .iter()
        .map(|package| {
            (
                package.declaration.name.as_str(),
                package_source_label(&package.declaration),
            )
        })
        .collect();
    let package_targets: std::collections::BTreeMap<&str, Vec<String>> = project
        .packages
        .iter()
        .map(|package| {
            (
                package.declaration.name.as_str(),
                package_target_names(package, project),
            )
        })
        .collect();
    let target_outputs: std::collections::BTreeMap<&str, Vec<String>> = project
        .manifest
        .targets
        .iter()
        .map(|target| (target.name.as_str(), target.outputs.clone()))
        .collect();

    let mut lines = Vec::new();
    let mut structured_updates = Vec::new();

    if let Some(previous) = previous {
        for source in &updated.source {
            let previous_revision = previous
                .source
                .iter()
                .find(|locked_source| locked_source.name == source.name)
                .and_then(|locked_source| locked_source.revision.as_deref());
            let updated_revision = source.revision.as_deref();
            if previous_revision != updated_revision {
                lines.push(format!(
                    "Updated source {} revision {} -> {}",
                    source.name,
                    previous_revision.unwrap_or("none"),
                    updated_revision.unwrap_or("none")
                ));
            }
        }
    }

    for package in &updated.package {
        if let Some(selected_package) = selected_package {
            if package.name != selected_package {
                continue;
            }
        }
        let Some(previous_package) = previous_packages.get(package.name.as_str()) else {
            lines.push(format!(
                "Updated package {} (new) -> {}",
                package.name, package.version
            ));
            continue;
        };
        let version_changed = previous_package.version != package.version;
        let artifact_changed = previous_package.artifact_hash != package.artifact_hash;
        let tree_changed = previous_package.tree_hash != package.tree_hash;
        if !version_changed && !artifact_changed && !tree_changed {
            continue;
        }

        let source = package_sources
            .get(package.name.as_str())
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let exports = package.exports.clone();
        let targets = package_targets
            .get(package.name.as_str())
            .cloned()
            .unwrap_or_default();
        let rendered_files: Vec<String> = targets
            .iter()
            .flat_map(|target_name| {
                target_outputs
                    .get(target_name.as_str())
                    .into_iter()
                    .flatten()
            })
            .cloned()
            .collect();
        let dependents = package_dependents(project, package.name.as_str());

        if version_changed {
            lines.push(format!(
                "Updated package {} {} -> {}",
                package.name, previous_package.version, package.version
            ));
        } else {
            lines.push(format!(
                "Refreshed package {} at {} (registry content changed)",
                package.name, package.version
            ));
        }
        lines.push(format!("  source: {source}"));
        lines.push(format!("  exports affected: {}", join_or_none(&exports)));
        lines.push(format!("  targets affected: {}", join_or_none(&targets)));
        lines.push(format!(
            "  rendered files affected: {}",
            join_or_none(&rendered_files)
        ));
        if !dependents.is_empty() {
            lines.push(format!(
                "  dependent packages affected: {}",
                join_or_none(&dependents)
            ));
        }
        lines.push("  warnings: none".to_string());
        structured_updates.push(serde_json::json!({
            "name": package.name,
            "from_version": previous_package.version,
            "to_version": package.version,
            "artifact_hash_changed": artifact_changed,
            "tree_hash_changed": tree_changed,
            "source": source,
            "exports_affected": exports,
            "targets_affected": targets,
            "rendered_files_affected": rendered_files,
            "dependent_packages_affected": dependents,
            "warnings": [],
        }));
    }

    Ok(UpdateSummaryReport {
        lines,
        updated_packages: structured_updates,
    })
}

fn package_dependents(
    project: &pray_core::resolve::ResolvedProject,
    selected_package: &str,
) -> Vec<String> {
    project
        .packages
        .iter()
        .filter(|package| {
            package
                .spec
                .dependencies
                .iter()
                .any(|dependency| dependency.name == selected_package)
        })
        .map(|package| package.declaration.name.clone())
        .collect()
}

fn package_source_label(declaration: &pray_core::manifest::ManifestPackage) -> String {
    if let Some(path) = &declaration.path {
        return format!("path:{path}");
    }
    if let Some(source) = &declaration.source {
        return format!("source:{source}");
    }
    "default".to_string()
}

fn package_target_names(
    package: &pray_core::resolve::ResolvedPackage,
    project: &pray_core::resolve::ResolvedProject,
) -> Vec<String> {
    if !package.declaration.targets.is_empty() {
        return package.declaration.targets.clone();
    }
    project
        .manifest
        .targets
        .iter()
        .map(|target| target.name.clone())
        .collect()
}

fn join_or_none(values: &[String]) -> String {
    if values.is_empty() {
        return "none".to_string();
    }
    values.join(", ")
}

fn format_marker_comments(text: &str) -> String {
    let lines: Vec<String> = text
        .split('\n')
        .map(|line| match canonical_marker_line(line) {
            Some(marker) => marker,
            None => line.to_string(),
        })
        .collect();
    let mut output = lines.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

fn canonical_marker_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let remainder = trimmed.strip_prefix("<!--")?.trim_start();
    let remainder = remainder.strip_prefix("pray:")?;
    let content = remainder.strip_suffix("-->")?.trim();
    if content == "0 ignore-comments" {
        return Some("<!-- pray:0 ignore-comments -->".to_string());
    }
    if content
        .chars()
        .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
    {
        return Some(format!("<!-- pray:{content} -->"));
    }
    None
}

fn render_tree_node(
    package: &pray_core::resolve::ResolvedPackage,
    package_map: &std::collections::BTreeMap<String, &pray_core::resolve::ResolvedPackage>,
    depth: usize,
    ancestry: &mut std::collections::BTreeSet<String>,
    lines: &mut Vec<String>,
) {
    let indent = "  ".repeat(depth);
    lines.push(format!(
        "{indent}{} {}",
        package.declaration.name, package.spec.version
    ));
    if !ancestry.insert(package.declaration.name.clone()) {
        return;
    }

    for dependency in &package.spec.dependencies {
        if let Some(resolved) = package_map.get(&dependency.name) {
            if ancestry.contains(&resolved.declaration.name) {
                lines.push(format!(
                    "{}  {} {} (cycle)",
                    indent, resolved.declaration.name, resolved.spec.version
                ));
            } else {
                render_tree_node(resolved, package_map, depth + 1, ancestry, lines);
            }
        } else {
            lines.push(format!(
                "{}  {} {} (unresolved)",
                indent, dependency.name, dependency.constraint
            ));
        }
    }

    ancestry.remove(&package.declaration.name);
}

fn package_source_summary(package: &pray_core::resolve::ResolvedPackage) -> String {
    package
        .declaration
        .path
        .as_ref()
        .map(|path| format!("path:{path}"))
        .or_else(|| {
            package
                .declaration
                .source
                .as_ref()
                .map(|source| format!("source:{source}"))
        })
        .unwrap_or_else(|| format!("root:{}", package.root.display()))
}

pub(crate) fn locked_package<'a>(
    lockfile: &'a Lockfile,
    package: &pray_core::resolve::ResolvedPackage,
) -> Option<&'a pray_core::lockfile::LockedPackage> {
    lockfile.package.iter().find(|record| {
        record.name == package.declaration.name
            && record.source.as_deref() == package.declaration.source.as_deref()
    })
}

fn format_list(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values.join(", ")
    }
}
