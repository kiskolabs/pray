mod auth_client;
mod server;

use auth_client::{
    current_signer as current_signer_from_session, login_with_passkey, login_with_ssh_agent,
};
use pray_core::auth::RegistryAuthStore;
use pray_core::hashing::{normalize_line_endings, sha256_prefixed};
use pray_core::lockfile::{read_lockfile, write_lockfile, Lockfile};
use pray_core::manifest::parse_manifest;
use pray_core::registry::{
    registry_artifact_signature, submit_confession, ConfessionSubmission, RegistryIndex,
    RegistryPackageMetadata, RegistryPackageVersion,
};
use pray_core::render::{render_project, write_rendered_targets};
use pray_core::resolve::{resolve_project, ResolvedProject};
use pray_core::verify::{drift_project, format_verification_report, verify_project};
use pray_core::{PrayError, PrayResult};
use pray_transport::{
    ArtifactRef, PeerConfig, PeerInfo, SyncDirection, TransportError, TransportRegistry, TrustLevel,
};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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
    let command = parse_command(arguments)?;
    match command {
        Command::Manifest => manifest_command(),
        Command::Init { targets } => init_command(targets),
        Command::Add {
            name,
            constraint,
            path,
        } => add_command(name, constraint, path),
        Command::Remove { name } => remove_command(name),
        Command::Update { package, major } => update_command(package, major),
        Command::Install {
            locked,
            frozen,
            offline,
        } => install_command(locked, frozen, offline),
        Command::Plan => plan_command(),
        Command::Apply => apply_command(),
        Command::Render { check } => render_command(check),
        Command::Verify { strict } => verify_command(strict),
        Command::Drift { semantic } => drift_command(semantic),
        Command::Format => format_command(),
        Command::Package => package_command(),
        Command::Publish { roots } => publish_command(roots),
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
        Command::Serve { root, host, port } => serve_command(root, host, port),
        Command::Confess {
            package,
            version,
            accepted,
            rejected,
            note,
            url,
        } => confess_command(package, version, accepted, rejected, note, url),
        Command::Vendor => vendor_command(),
        Command::Clean => clean_command(),
        Command::Tree => tree_command(),
        Command::Sync { root, peers } => sync_command(root, peers),
    }
}

enum Command {
    Manifest,
    Init {
        targets: Vec<String>,
    },
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
    },
    Render {
        check: bool,
    },
    Plan,
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
    },
    Login {
        servers: Vec<String>,
        email: String,
        credential_id: Option<String>,
        passkey_key: Option<PathBuf>,
        public_key: Option<PathBuf>,
        ssh_agent: bool,
    },
    Serve {
        root: PathBuf,
        host: String,
        port: u16,
    },
    Confess {
        package: String,
        version: Option<String>,
        accepted: bool,
        rejected: bool,
        note: Option<String>,
        url: Option<String>,
    },
    Vendor,
    Clean,
    Tree,
    Sync {
        root: PathBuf,
        peers: Vec<String>,
    },
}

fn parse_command(arguments: Vec<String>) -> PrayResult<Command> {
    let check = arguments.iter().any(|argument| argument == "--check");
    let strict = arguments.iter().any(|argument| argument == "--strict");
    let semantic = arguments.iter().any(|argument| argument == "--semantic");
    let mut iter = arguments.into_iter();
    let command = iter.next().unwrap_or_else(|| "help".to_string());
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
        "render" => Ok(Command::Render { check }),
        "plan" => Ok(Command::Plan),
        "apply" => Ok(Command::Apply),
        "verify" => Ok(Command::Verify { strict }),
        "drift" => Ok(Command::Drift { semantic }),
        "format" => Ok(Command::Format),
        "package" => Ok(Command::Package),
        "publish" => parse_publish_command(iter),
        "login" => parse_login_command(iter),
        "serve" => parse_serve_command(iter),
        "confess" => parse_confess_command(iter),
        "vendor" => Ok(Command::Vendor),
        "clean" => Ok(Command::Clean),
        "tree" => Ok(Command::Tree),
        "sync" => parse_sync_command(iter),
        "help" | "-h" | "--help" => {
            print_help();
            std::process::exit(0);
        }
        other => Err(PrayError::Unsupported(format!("unknown command: {other}"))),
    }
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

fn add_command(name: String, constraint: Option<String>, path: Option<String>) -> PrayResult<()> {
    let manifest_path = manifest_path();
    let manifest_text = fs::read_to_string(&manifest_path)?;
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
    let manifest_text = fs::read_to_string(&manifest_path)?;
    let manifest = parse_manifest(&manifest_text)?;
    if !manifest.packages.iter().any(|package| package.name == name) {
        return Err(PrayError::Manifest(format!("package {name} not found")));
    }

    fs::write(
        manifest_path,
        remove_manifest_statement(&manifest_text, &name),
    )?;
    install_command(false, false, false)
}

fn update_command(package: Option<String>, major: bool) -> PrayResult<()> {
    if major && package.is_none() {
        return Err(PrayError::Unsupported(
            "major updates require a package name".to_string(),
        ));
    }

    let project = resolve_project(&manifest_path())?;
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

    let previous_lockfile = read_lockfile(&lockfile_path()).ok();
    install_command(false, false, false)?;
    let updated_lockfile = read_lockfile(&lockfile_path())?;
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
    print_update_summary(
        previous_lockfile.as_ref(),
        &merged_lockfile,
        package.as_deref(),
        &project,
    )?;
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
    for argument in arguments {
        match argument.as_str() {
            "--major" => major = true,
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
    Ok(Command::Update { package, major })
}

fn parse_publish_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<Command> {
    let mut roots = Vec::new();
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
    if roots.is_empty() {
        return Err(PrayError::Unsupported(
            "publish requires at least one --root PATH".to_string(),
        ));
    }
    Ok(Command::Publish { roots })
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
    Ok(Command::Serve { root, host, port })
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
    let mut version = None;
    let mut accepted = false;
    let mut rejected = false;
    let mut note = None;
    let mut url = None;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
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
                if package.is_none() {
                    package = Some(other.to_string());
                } else {
                    return Err(PrayError::Unsupported(format!(
                        "unexpected confess argument: {other}"
                    )));
                }
            }
        }
    }
    let package = package
        .ok_or_else(|| PrayError::Unsupported("confess requires a package name".to_string()))?;
    if accepted == rejected {
        return Err(PrayError::Unsupported(
            "confess requires exactly one of --accepted or --rejected".to_string(),
        ));
    }
    Ok(Command::Confess {
        package,
        version,
        accepted,
        rejected,
        note,
        url,
    })
}

fn install_command(locked: bool, frozen: bool, offline: bool) -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    if offline {
        ensure_offline_ready(&project)?;
    }
    let rendered = render_project(&project)?;
    let lockfile_path = lockfile_path();
    if locked {
        let lockfile = ensure_existing_lockfile(&lockfile_path)?;
        ensure_lockfile_current(&project, &rendered, &lockfile)?;
        if frozen {
            ensure_rendered_outputs_current(&project, &rendered)?;
            return Ok(());
        }
        write_rendered_targets(&project, &rendered)?;
        return Ok(());
    }

    let lockfile = build_lockfile(&project, &rendered)?;
    write_lockfile_if_changed(&lockfile_path, &lockfile)?;
    write_rendered_targets(&project, &rendered)?;
    Ok(())
}

fn plan_command() -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    let rendered = render_project(&project)?;
    let lockfile = build_lockfile(&project, &rendered)?;
    let mut lines = Vec::new();

    match read_lockfile(&lockfile_path()) {
        Ok(existing) if existing.canonicalized() == lockfile.canonicalized() => {
            lines.push("Prayfile.lock unchanged".to_string());
        }
        Ok(_) => lines.push("Prayfile.lock would be updated".to_string()),
        Err(_) => lines.push("Prayfile.lock would be created".to_string()),
    }

    for target in &rendered {
        let path = project.project_root.join(&target.path);
        let status = match fs::read_to_string(&path) {
            Ok(existing) if existing == target.content => "unchanged",
            Ok(_) => "would be updated",
            Err(_) => "would be written",
        };
        lines.push(format!("{} {status}", target.path.display()));
    }

    println!("{}", lines.join("\n"));
    Ok(())
}

fn apply_command() -> PrayResult<()> {
    install_command(false, false, false)
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

fn serve_command(root: PathBuf, host: String, port: u16) -> PrayResult<()> {
    server::run_server(root, host, port)
}

fn confess_command(
    package: String,
    version: Option<String>,
    accepted: bool,
    rejected: bool,
    note: Option<String>,
    url: Option<String>,
) -> PrayResult<()> {
    if accepted == rejected {
        return Err(PrayError::Unsupported(
            "confess requires exactly one of --accepted or --rejected".to_string(),
        ));
    }

    let project = resolve_project(&manifest_path())?;
    let package_resolution = project
        .packages
        .iter()
        .find(|resolved_package| resolved_package.declaration.name == package)
        .ok_or_else(|| PrayError::Resolution(format!("package {package} not found")))?;

    let resolved_version = version.unwrap_or_else(|| package_resolution.spec.version.clone());
    if resolved_version != package_resolution.spec.version {
        return Err(PrayError::Resolution(format!(
            "package {package} version {} does not match resolved version {}",
            resolved_version, package_resolution.spec.version
        )));
    }

    let source_name = package_resolution
        .declaration
        .source
        .as_ref()
        .ok_or_else(|| PrayError::Resolution(format!("package {package} is missing a source")))?;
    let source_url = if let Some(url) = url {
        url
    } else {
        project
            .manifest
            .sources
            .iter()
            .find(|source| source.name == *source_name)
            .map(|source| source.url.clone())
            .ok_or_else(|| PrayError::Resolution(format!("unknown source: {source_name}")))?
    };

    let lockfile = read_lockfile(&lockfile_path()).ok();
    let lockfile_reference = lockfile
        .as_ref()
        .and_then(|lockfile| lockfile.file_hash().ok());
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| PrayError::Resolution(error.to_string()))?
        .as_secs()
        .to_string();
    let mut confession = ConfessionSubmission {
        package: package.clone(),
        version: resolved_version,
        status: if accepted {
            "accepted".to_string()
        } else {
            "rejected".to_string()
        },
        note,
        lockfile: lockfile_reference,
        distribution_point: Some(source_url.clone()),
        signer: Some(current_signer()),
        timestamp: Some(timestamp),
        signature: None,
    };
    let signature_payload =
        serde_json::to_vec(&confession).map_err(|error| PrayError::Manifest(error.to_string()))?;
    confession.signature = Some(sha256_prefixed(&signature_payload));
    submit_confession(&source_url, &confession)?;
    println!(
        "Confession submitted for {} {}",
        confession.package, confession.version
    );
    Ok(())
}

fn current_signer() -> String {
    let session_root = workspace_root();
    if let Some(email) = current_signer_from_session(&session_root) {
        return email;
    }

    if let Ok(token) = std::env::var("PRAY_SESSION_TOKEN") {
        let auth_root = std::env::var("PRAY_AUTH_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| session_root.clone());
        if let Ok(store) = RegistryAuthStore::open(&auth_root) {
            if let Ok(Some(session)) = store.resolve_session(&token) {
                return session.email;
            }
        }
    }

    std::env::var("PRAY_SIGNER")
        .or_else(|_| std::env::var("USER"))
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

fn current_timestamp() -> PrayResult<String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| PrayError::Resolution(error.to_string()))
        .map(|duration| duration.as_secs().to_string())
}

fn format_command() -> PrayResult<()> {
    let lockfile = read_lockfile(&lockfile_path())?;
    for target in &lockfile.target {
        for output in &target.outputs {
            let path = Path::new(output);
            let original = fs::read_to_string(path)?;
            let formatted = format_marker_comments(&normalize_line_endings(&original));
            if formatted != original {
                fs::write(path, formatted)?;
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

fn publish_command(roots: Vec<PathBuf>) -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    let signer = current_signer();
    let published_at = current_timestamp()?;
    for root in roots {
        publish_to_root(&project, &signer, &published_at, &root)?;
    }
    Ok(())
}

fn publish_to_root(
    project: &ResolvedProject,
    signer: &str,
    published_at: &str,
    root: &Path,
) -> PrayResult<()> {
    let mut registry_index = load_registry_index(root)?;
    let mut package_names = registry_index
        .packages
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();

    for package in &project.packages {
        let archive_bytes = build_package_archive_bytes(package)?;
        let artifact_path =
            registry_artifact_path(&package.declaration.name, &package.spec.version);
        let artifact_output_path = root.join(&artifact_path);
        if let Some(parent) = artifact_output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&artifact_output_path, &archive_bytes)?;

        let metadata_path = registry_metadata_path(root, &package.declaration.name);
        let mut metadata =
            load_registry_package_metadata(&metadata_path, &package.declaration.name)?;
        let signature = registry_artifact_signature(&archive_bytes, &package.tree_hash, signer);
        let version_entry = RegistryPackageVersion {
            version: package.spec.version.clone(),
            artifact: artifact_path,
            artifact_hash: Some(sha256_prefixed(&archive_bytes)),
            tree_hash: Some(package.tree_hash.clone()),
            yanked: false,
            targets: package.spec.targets.clone(),
            exports: package.spec.exports.keys().cloned().collect(),
            signer: Some(signer.to_string()),
            published_at: Some(published_at.to_string()),
            signature: Some(signature),
        };
        metadata
            .versions
            .retain(|entry| entry.version != version_entry.version);
        metadata.versions.push(version_entry);
        write_registry_package_metadata(&metadata_path, &metadata)?;
        package_names.insert(package.declaration.name.clone());
    }

    registry_index.packages = package_names.into_iter().collect();
    write_registry_index(root, &registry_index)?;
    Ok(())
}

fn sync_command(root: PathBuf, peers: Vec<String>) -> PrayResult<()> {
    let peer_sources = if peers.is_empty() {
        load_sync_peers(&root)?
    } else {
        peers
    };

    if peer_sources.is_empty() {
        return Err(PrayError::Unsupported(
            "no federation peers configured".to_string(),
        ));
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| {
            PrayError::Unsupported(format!("failed to start sync runtime: {error}"))
        })?;

    let summary = runtime.block_on(async { synchronize_registry(&root, peer_sources).await })?;

    println!(
        "Synchronized {} package(s) from {} peer(s)",
        summary.packages, summary.peers
    );
    Ok(())
}

struct SyncSummary {
    peers: usize,
    packages: usize,
}

async fn synchronize_registry(root: &Path, peer_sources: Vec<String>) -> PrayResult<SyncSummary> {
    let registry = TransportRegistry::new();
    let mut discovered_peers = BTreeSet::new();
    let mut package_versions_by_name: BTreeMap<String, BTreeMap<String, RegistryPackageVersion>> =
        BTreeMap::new();
    let mut peer_count = 0usize;

    for peer_source in peer_sources {
        if !discovered_peers.insert(peer_source.clone()) {
            continue;
        }
        peer_count += 1;
        let peer = federation_peer_config(&peer_source);
        let transport = registry.create(&peer).map_err(map_transport_error)?;
        let discovery = transport
            .fetch_discovery(&peer)
            .await
            .map_err(map_transport_error)?;
        if discovery.spec != "pray-federation-v1" {
            return Err(PrayError::Resolution(format!(
                "peer {peer_source} does not speak the pray federation protocol"
            )));
        }

        let index = transport
            .fetch_index(&peer, None)
            .await
            .map_err(map_transport_error)?;
        if index.spec != "prayfile-distribution-1" {
            return Err(PrayError::Resolution(format!(
                "peer {peer_source} returned unsupported registry index spec: {}",
                index.spec
            )));
        }

        for package_summary in index.packages {
            let metadata = transport
                .fetch_package(&peer, &package_summary.name)
                .await
                .map_err(map_transport_error)?;
            if metadata.name != package_summary.name {
                return Err(PrayError::Resolution(format!(
                    "peer {peer_source} returned mismatched package metadata for {}",
                    package_summary.name
                )));
            }
            sync_package_from_peer(
                root,
                &peer,
                transport.as_ref(),
                metadata,
                &mut package_versions_by_name,
            )
            .await?;
        }
    }

    let mut local_index = load_registry_index(root)?;
    let mut package_names: BTreeSet<String> = local_index.packages.into_iter().collect();
    for (package_name, version_map) in &package_versions_by_name {
        write_synced_package_metadata(root, package_name, version_map)?;
        package_names.insert(package_name.clone());
    }
    local_index.packages = package_names.into_iter().collect();
    write_registry_index(root, &local_index)?;

    Ok(SyncSummary {
        peers: peer_count,
        packages: package_versions_by_name.len(),
    })
}

async fn sync_package_from_peer(
    root: &Path,
    peer: &PeerConfig,
    transport: &dyn pray_transport::TransportAdapter,
    metadata: pray_transport::PackageMetadata,
    package_versions_by_name: &mut BTreeMap<String, BTreeMap<String, RegistryPackageVersion>>,
) -> PrayResult<()> {
    let package_versions = package_versions_by_name
        .entry(metadata.name.clone())
        .or_insert_with(|| load_local_package_versions(root, &metadata.name).unwrap_or_default());

    for version in metadata.versions {
        let local_version = sync_package_version_from_transport(&version)?;
        if let Some(existing_version) = package_versions.get(&local_version.version) {
            if existing_version == &local_version {
                continue;
            }
            return Err(PrayError::Integrity(format!(
                "conflicting metadata for package {} version {}",
                metadata.name, local_version.version
            )));
        }

        let artifact_hash = local_version.artifact_hash.as_ref().ok_or_else(|| {
            PrayError::Integrity(format!(
                "federation package {} {} is missing an artifact hash",
                metadata.name, local_version.version
            ))
        })?;
        let artifact = ArtifactRef {
            name: metadata.name.clone(),
            version: local_version.version.clone(),
            url: local_version.artifact.clone(),
            hash: artifact_hash.clone(),
        };
        let bytes = transport
            .fetch_artifact(peer, &artifact)
            .await
            .map_err(map_transport_error)?;
        let computed_hash = sha256_prefixed(&bytes);
        if &computed_hash != artifact_hash {
            return Err(PrayError::Integrity(format!(
                "artifact hash mismatch for {} {}",
                metadata.name, local_version.version
            )));
        }
        if let Some(signature) = local_version.signature.as_ref() {
            if let (Some(tree_hash), Some(signer)) = (
                local_version.tree_hash.as_ref(),
                local_version.signer.as_ref(),
            ) {
                let expected_signature = registry_artifact_signature(&bytes, tree_hash, signer);
                if &expected_signature != signature {
                    return Err(PrayError::Integrity(format!(
                        "signature mismatch for {} {}",
                        metadata.name, local_version.version
                    )));
                }
            }
        }

        write_synced_artifact(root, &local_version.artifact, &bytes)?;
        package_versions.insert(local_version.version.clone(), local_version);
    }

    Ok(())
}

fn load_local_package_versions(
    root: &Path,
    package_name: &str,
) -> PrayResult<BTreeMap<String, RegistryPackageVersion>> {
    let metadata_path = registry_metadata_path(root, package_name);
    let metadata = load_registry_package_metadata(&metadata_path, package_name)?;
    Ok(metadata
        .versions
        .into_iter()
        .map(|version| (version.version.clone(), version))
        .collect())
}

fn write_synced_package_metadata(
    root: &Path,
    package_name: &str,
    versions: &BTreeMap<String, RegistryPackageVersion>,
) -> PrayResult<()> {
    let metadata = RegistryPackageMetadata {
        name: package_name.to_string(),
        versions: versions.values().cloned().collect(),
    };
    let metadata_path = registry_metadata_path(root, package_name);
    write_registry_package_metadata(&metadata_path, &metadata)
}

fn write_synced_artifact(root: &Path, artifact_path: &str, bytes: &[u8]) -> PrayResult<()> {
    let path = root.join(artifact_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)?;
    Ok(())
}

fn federation_peer_config(peer_source: &str) -> PeerConfig {
    PeerConfig {
        name: peer_source.to_string(),
        transport: "http".to_string(),
        url: Some(peer_source.to_string()),
        trust: TrustLevel::Full,
        direction: SyncDirection::Pull,
        config: serde_json::json!({}),
    }
}

fn load_sync_peers(root: &Path) -> PrayResult<Vec<String>> {
    let path = root.join("v1/peers.json");
    let text = fs::read_to_string(&path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            PrayError::Unsupported("no federation peers configured".to_string())
        } else {
            PrayError::from(error)
        }
    })?;
    let peers: Vec<PeerInfo> = serde_json::from_str(&text).map_err(|error| PrayError::Parse {
        kind: "peer list",
        message: error.to_string(),
    })?;
    let mut peer_sources = Vec::new();
    for peer in peers {
        if peer.url.trim().is_empty() {
            return Err(PrayError::Resolution(
                "peer list contains an entry with an empty url".to_string(),
            ));
        }
        peer_sources.push(peer.url);
    }
    Ok(peer_sources)
}

fn sync_package_version_from_transport(
    version: &pray_transport::PackageVersion,
) -> PrayResult<RegistryPackageVersion> {
    if version.version.trim().is_empty() {
        return Err(PrayError::Resolution(
            "federation package version missing version string".to_string(),
        ));
    }
    if version.artifact.trim().is_empty() {
        return Err(PrayError::Resolution(format!(
            "federation package version {} missing artifact path",
            version.version
        )));
    }
    if version.artifact_hash.trim().is_empty() {
        return Err(PrayError::Integrity(format!(
            "federation package version {} missing artifact hash",
            version.version
        )));
    }

    let signer = version
        .publisher
        .as_ref()
        .map(|publisher| publisher.id.clone())
        .filter(|signer| !signer.trim().is_empty())
        .or_else(|| {
            version
                .signature
                .as_ref()
                .map(|signature| signature.public_key.clone())
                .filter(|signer| !signer.trim().is_empty())
        });

    Ok(RegistryPackageVersion {
        version: version.version.clone(),
        artifact: version.artifact.clone(),
        artifact_hash: Some(version.artifact_hash.clone()),
        tree_hash: if version.tree_hash.trim().is_empty() {
            None
        } else {
            Some(version.tree_hash.clone())
        },
        yanked: version.yanked,
        targets: version.targets.clone(),
        exports: version.exports.clone(),
        signer,
        published_at: Some(version.published_at.clone()),
        signature: version
            .signature
            .as_ref()
            .map(|signature| signature.signature.clone()),
    })
}

fn map_transport_error(error: TransportError) -> PrayError {
    match error {
        TransportError::InvalidResponse(message) => PrayError::Parse {
            kind: "federation response",
            message,
        },
        TransportError::Json(error) => PrayError::Parse {
            kind: "federation response",
            message: error.to_string(),
        },
        TransportError::Io(error) => PrayError::Io(error),
        other => PrayError::Resolution(other.to_string()),
    }
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

fn materialize_package_directory(
    package: &pray_core::resolve::ResolvedPackage,
    output_directory: &Path,
) -> PrayResult<()> {
    if output_directory.exists() {
        remove_path_if_exists(output_directory)?;
    }
    fs::create_dir_all(output_directory)?;
    let metadata = serde_json::json!({
        "name": package.declaration.name,
        "version": package.spec.version,
        "tree_hash": package.tree_hash,
        "artifact_hash": package.artifact_hash,
        "exports": package.spec.exports.keys().cloned().collect::<Vec<_>>(),
        "files": package.spec.files,
        "dependencies": package
            .spec
            .dependencies
            .iter()
            .map(|dependency| serde_json::json!({
                "name": dependency.name,
                "constraint": dependency.constraint,
                "optional": dependency.optional,
            }))
            .collect::<Vec<_>>(),
    });
    fs::write(
        output_directory.join("metadata.json"),
        serde_json::to_string_pretty(&metadata)
            .map_err(|error| PrayError::Manifest(error.to_string()))?,
    )?;
    copy_prayspec_file(&package.root, output_directory)?;

    for file in &package.spec.files {
        copy_package_file(&package.root, output_directory, file)?;
    }
    Ok(())
}

fn write_package_archive(
    package: &pray_core::resolve::ResolvedPackage,
    output_path: &Path,
) -> PrayResult<()> {
    if output_path.exists() {
        remove_path_if_exists(output_path)?;
    }
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let archive_bytes = build_package_archive_bytes(package)?;
    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&archive_bytes)?;
    output_file.flush()?;
    Ok(())
}

fn build_package_archive_bytes(
    package: &pray_core::resolve::ResolvedPackage,
) -> PrayResult<Vec<u8>> {
    let metadata = package_metadata(package)?;
    let mut tar_bytes = Vec::new();
    {
        let mut archive = tar::Builder::new(&mut tar_bytes);
        append_archive_file(
            &mut archive,
            Path::new("metadata.json"),
            metadata.as_bytes(),
        )?;
        let prayspec_path = find_prayspec_file(&package.root)?;
        let prayspec_name = prayspec_path
            .file_name()
            .ok_or_else(|| PrayError::Integrity("missing prayspec filename".to_string()))?;
        let prayspec_bytes = fs::read(&prayspec_path)?;
        append_archive_file(&mut archive, Path::new(prayspec_name), &prayspec_bytes)?;
        for file in &package.spec.files {
            let content = read_package_file_bytes(&package.root, file)?;
            append_archive_file(&mut archive, Path::new(file), &content)?;
        }
        archive.finish()?;
    }

    let mut output = Vec::new();
    zstd::stream::copy_encode(&tar_bytes[..], &mut output, 0)?;
    Ok(output)
}

fn load_registry_index(root: &Path) -> PrayResult<RegistryIndex> {
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

fn load_registry_package_metadata(
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

fn write_registry_index(root: &Path, index: &RegistryIndex) -> PrayResult<()> {
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

fn write_registry_package_metadata(
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

fn registry_metadata_path(root: &Path, package_name: &str) -> PathBuf {
    root.join("v1/packages")
        .join(package_name)
        .with_extension("json")
}

fn registry_artifact_path(package_name: &str, version: &str) -> String {
    let artifact_name = format!("{}-{}.praypkg", package_name.replace('/', "-"), version);
    format!("v1/artifacts/{package_name}/{version}/{artifact_name}")
}

fn package_metadata(package: &pray_core::resolve::ResolvedPackage) -> PrayResult<String> {
    serde_json::to_string_pretty(&serde_json::json!({
        "name": package.declaration.name,
        "version": package.spec.version,
        "tree_hash": package.tree_hash,
        "artifact_hash": package.artifact_hash,
        "exports": package.spec.exports.keys().cloned().collect::<Vec<_>>(),
        "files": package.spec.files,
        "dependencies": package
            .spec
            .dependencies
            .iter()
            .map(|dependency| serde_json::json!({
                "name": dependency.name,
                "constraint": dependency.constraint,
                "optional": dependency.optional,
            }))
            .collect::<Vec<_>>(),
    }))
    .map_err(|error| PrayError::Manifest(error.to_string()))
}

fn append_archive_file(
    archive: &mut tar::Builder<&mut Vec<u8>>,
    path: &Path,
    contents: &[u8],
) -> PrayResult<()> {
    let mut header = tar::Header::new_gnu();
    header.set_size(contents.len() as u64);
    header.set_mode(0o644);
    header.set_mtime(0);
    header.set_uid(0);
    header.set_gid(0);
    header.set_cksum();
    archive.append_data(&mut header, path, contents)?;
    Ok(())
}

fn read_package_file_bytes(source_root: &Path, relative_path: &str) -> PrayResult<Vec<u8>> {
    let relative = Path::new(relative_path);
    validate_package_relative_path(relative)?;
    let source = source_root.join(relative);
    if !source.exists() {
        return Err(PrayError::Integrity(format!(
            "package file missing: {}",
            relative_path
        )));
    }
    if source.is_dir() {
        return Err(PrayError::Integrity(format!(
            "package file is a directory: {}",
            relative_path
        )));
    }
    Ok(fs::read(source)?)
}

fn find_prayspec_file(root: &Path) -> PrayResult<PathBuf> {
    let mut prayspec_files = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some("prayspec") {
            prayspec_files.push(path);
        }
    }
    match prayspec_files.len() {
        1 => Ok(prayspec_files.remove(0)),
        0 => Err(PrayError::Integrity(format!(
            "no prayspec file found in {:?}",
            root
        ))),
        _ => Err(PrayError::Integrity(format!(
            "multiple prayspec files found in {:?}",
            root
        ))),
    }
}

fn copy_prayspec_file(source_root: &Path, archive_root: &Path) -> PrayResult<()> {
    let prayspec_path = find_prayspec_file(source_root)?;
    let prayspec_name = prayspec_path
        .file_name()
        .ok_or_else(|| PrayError::Integrity("missing prayspec filename".to_string()))?;
    let destination = archive_root.join(prayspec_name);
    fs::copy(prayspec_path, destination)?;
    Ok(())
}

fn render_command(check_only: bool) -> PrayResult<()> {
    let project = resolve_project(&manifest_path())?;
    let rendered = render_project(&project)?;
    if check_only {
        ensure_rendered_outputs_current(&project, &rendered)?;
        return Ok(());
    }
    let lockfile = build_lockfile(&project, &rendered)?;
    write_lockfile(&lockfile_path(), &lockfile)?;
    write_rendered_targets(&project, &rendered)?;
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
        &project.manifest.sources,
        &project.manifest.targets,
        rendered,
        &project.packages,
    ))
}

fn load_manifest() -> PrayResult<pray_core::manifest::Manifest> {
    let text = fs::read_to_string(manifest_path())?;
    parse_manifest(&text)
}

fn workspace_root() -> PathBuf {
    manifest_path()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn manifest_path() -> PathBuf {
    PathBuf::from("Prayfile")
}

fn lockfile_path() -> PathBuf {
    PathBuf::from("Prayfile.lock")
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
        return Err(PrayError::Verify("missing Prayfile.lock".to_string()));
    }
    read_lockfile(path)
}

fn ensure_lockfile_current(
    project: &pray_core::resolve::ResolvedProject,
    rendered: &[pray_core::render::RenderedTarget],
    existing: &Lockfile,
) -> PrayResult<()> {
    let current = build_lockfile(project, rendered)?;
    if current.canonicalized() != existing.canonicalized() {
        return Err(PrayError::Verify("lockfile needs update".to_string()));
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
            return Err(PrayError::Render(format!("{} is stale", path.display())));
        }
    }
    Ok(())
}

fn write_lockfile_if_changed(path: &Path, lockfile: &Lockfile) -> PrayResult<()> {
    if path.exists() {
        if let Ok(existing) = read_lockfile(path) {
            if existing.canonicalized() == lockfile.canonicalized() {
                return Ok(());
            }
        }
    }
    write_lockfile(path, lockfile)
}

fn ensure_offline_ready(project: &pray_core::resolve::ResolvedProject) -> PrayResult<()> {
    for declaration in &project.manifest.packages {
        if declaration.path.is_some() {
            continue;
        }
        if let Some(source_name) = &declaration.source {
            let source = project
                .manifest
                .sources
                .iter()
                .find(|candidate| candidate.name == *source_name)
                .ok_or_else(|| PrayError::Resolution(format!("unknown source: {source_name}")))?;
            if source.kind == "path" {
                continue;
            }
        }
        return Err(PrayError::Unsupported(
            "offline mode requires explicit local path packages".to_string(),
        ));
    }
    Ok(())
}

fn remove_path_if_exists(path: &Path) -> PrayResult<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => {
            fs::remove_dir_all(path)?;
            Ok(())
        }
        Ok(_) => {
            fs::remove_file(path)?;
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn package_archive_path(name: &str, version: &str) -> PathBuf {
    PathBuf::from(format!("{}-{}.praypkg", name.replace('/', "-"), version))
}

fn vendor_package_path(name: &str, version: &str) -> PathBuf {
    PathBuf::from(".pray/vendor")
        .join(name.replace('/', "-"))
        .join(version)
}

fn copy_package_file(
    source_root: &Path,
    archive_root: &Path,
    relative_path: &str,
) -> PrayResult<()> {
    let relative = Path::new(relative_path);
    validate_package_relative_path(relative)?;
    let source = source_root.join(relative);
    if !source.exists() {
        return Err(PrayError::Integrity(format!(
            "package file missing: {}",
            relative_path
        )));
    }
    if source.is_dir() {
        return Err(PrayError::Integrity(format!(
            "package file is a directory: {}",
            relative_path
        )));
    }
    let destination = archive_root.join(relative);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, destination)?;
    Ok(())
}

fn validate_package_relative_path(path: &Path) -> PrayResult<()> {
    if path.is_absolute() {
        return Err(PrayError::Integrity(format!(
            "package file path must be relative: {}",
            path.display()
        )));
    }
    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(PrayError::Integrity(format!(
                "package file path may not traverse upward: {}",
                path.display()
            )));
        }
    }
    Ok(())
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
            *package = previous_package.clone();
        }
    }
    merged
}

fn print_update_summary(
    previous: Option<&Lockfile>,
    updated: &Lockfile,
    selected_package: Option<&str>,
    project: &pray_core::resolve::ResolvedProject,
) -> PrayResult<()> {
    let previous_versions: std::collections::BTreeMap<&str, &str> = previous
        .into_iter()
        .flat_map(|lockfile| lockfile.package.iter())
        .map(|package| (package.name.as_str(), package.version.as_str()))
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
    for package in &updated.package {
        if let Some(selected_package) = selected_package {
            if package.name != selected_package {
                continue;
            }
        }
        let Some(previous_version) = previous_versions.get(package.name.as_str()) else {
            lines.push(format!(
                "Updated package {} (new) -> {}",
                package.name, package.version
            ));
            continue;
        };
        if *previous_version == package.version {
            continue;
        }

        let source = package_sources
            .get(package.name.as_str())
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let exports = updated
            .package
            .iter()
            .find(|locked_package| locked_package.name == package.name)
            .map(|locked_package| locked_package.exports.clone())
            .unwrap_or_default();
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

        lines.push(format!(
            "Updated package {} {} -> {}",
            package.name, previous_version, package.version
        ));
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
            "from_version": previous_version,
            "to_version": package.version,
            "source": source,
            "exports_affected": exports,
            "targets_affected": targets,
            "rendered_files_affected": rendered_files,
            "dependent_packages_affected": dependents,
            "warnings": [],
        }));
    }

    if lines.is_empty() {
        return Ok(());
    }

    println!("Update summary");
    for line in lines {
        println!("{line}");
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "updated_packages": structured_updates,
        }))
        .map_err(|error| PrayError::Manifest(error.to_string()))?
    );
    Ok(())
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

fn print_help() {
    println!("pray <command>");
    println!("  manifest");
    println!("  init [--targets tool_a,tool_b]");
    println!("  add <name> [constraint] [--path PATH]");
    println!("  remove <name>");
    println!("  update [package] [--major]");
    println!("  install [--locked|--frozen|--offline]");
    println!("  render [--check]");
    println!("  plan");
    println!("  apply");
    println!("  verify [--strict]");
    println!("  drift [--semantic]");
    println!("  format");
    println!("  package");
    println!("  publish --root PATH");
    println!("  login --server URL --email EMAIL [--credential-id ID --passkey-key PATH | --public-key PATH --ssh-agent]");
    println!("  serve [--root PATH] [--host HOST] [--port PORT]");
    println!("  sync [--root PATH] [--peer URL ...]");
    println!(
        "  confess <package> [--version VERSION] [--accepted|--rejected] [--note NOTE] [--url URL]"
    );
    println!("  vendor");
    println!("  clean");
    println!("  tree");
}
