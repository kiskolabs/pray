use pray_core::client_trust::{
    add_allowed_signing_key, check_compromised_keys, effective_trust_home,
    import_registry_trust, import_signing_keys_from_repository, list_policy, parse_compromised_feed,
    remove_allowed_signing_key, set_allow, set_require_signed_commit, show_policy_toml,
    TrustListScope, DEFAULT_COMPROMISED_KEYS_SOURCE,
};
use pray_core::resolve::git_source_cache_directory;
use pray_core::{PrayError, PrayResult};
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn run_trust_command(arguments: Vec<String>) -> PrayResult<()> {
    let mut iter = arguments.into_iter();
    let subcommand = iter
        .next()
        .ok_or_else(|| PrayError::Unsupported("trust requires a subcommand".into()))?;
    match subcommand.as_str() {
        "list" => trust_list_command(iter),
        "show" => trust_show_command(),
        "add-key" => trust_add_key_command(iter),
        "remove-key" | "revoke" => trust_remove_key_command(iter),
        "set-signed" => trust_set_signed_command(iter),
        "set-allow" => trust_set_allow_command(iter),
        "import-repo" => trust_import_repo_command(iter),
        "import-registry" => trust_import_registry_command(iter),
        "check" => trust_check_command(iter),
        other => Err(PrayError::Unsupported(format!(
            "unknown trust command: {other}"
        ))),
    }
}

fn trust_home() -> PrayResult<PathBuf> {
    effective_trust_home()
}

fn trust_list_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<()> {
    let mut scope = TrustListScope::All;
    let mut source_filter: Option<String> = None;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--global" => scope = TrustListScope::Global,
            "--local" => scope = TrustListScope::Local,
            value if !value.starts_with('-') => source_filter = Some(value.to_string()),
            other => {
                return Err(PrayError::Unsupported(format!(
                    "unknown trust list argument: {other}"
                )))
            }
        }
    }
    println!(
        "{}",
        list_policy(&trust_home()?, scope, source_filter.as_deref(),)?
    );
    Ok(())
}

fn trust_show_command() -> PrayResult<()> {
    println!("{}", show_policy_toml(&trust_home()?)?);
    Ok(())
}

fn trust_add_key_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<()> {
    let key = arguments
        .next()
        .ok_or_else(|| PrayError::Unsupported("trust add-key requires KEY".into()))?;
    let mut match_prefix = None;
    while let Some(argument) = arguments.next() {
        if argument == "--match-prefix" {
            match_prefix =
                Some(arguments.next().ok_or_else(|| {
                    PrayError::Unsupported("--match-prefix requires VALUE".into())
                })?);
        } else {
            return Err(PrayError::Unsupported(format!(
                "unknown trust add-key argument: {argument}"
            )));
        }
    }
    add_allowed_signing_key(&trust_home()?, &key, match_prefix.as_deref())?;
    Ok(())
}

fn trust_remove_key_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<()> {
    let key = arguments
        .next()
        .ok_or_else(|| PrayError::Unsupported("trust remove-key requires KEY".into()))?;
    let mut match_prefix = None;
    while let Some(argument) = arguments.next() {
        if argument == "--match-prefix" {
            match_prefix =
                Some(arguments.next().ok_or_else(|| {
                    PrayError::Unsupported("--match-prefix requires VALUE".into())
                })?);
        } else {
            return Err(PrayError::Unsupported(format!(
                "unknown trust remove-key argument: {argument}"
            )));
        }
    }
    remove_allowed_signing_key(&trust_home()?, &key, match_prefix.as_deref())?;
    Ok(())
}

fn trust_set_signed_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<()> {
    let mut match_prefix = None;
    let mut enabled = true;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--match-prefix" => {
                match_prefix = Some(arguments.next().ok_or_else(|| {
                    PrayError::Unsupported("--match-prefix requires VALUE".into())
                })?);
            }
            "--enabled" => {
                let value = arguments.next().ok_or_else(|| {
                    PrayError::Unsupported("--enabled requires true|false".into())
                })?;
                enabled = parse_bool_flag(&value)?;
            }
            other => {
                return Err(PrayError::Unsupported(format!(
                    "unknown trust set-signed argument: {other}"
                )))
            }
        }
    }
    let prefix = match_prefix.ok_or_else(|| {
        PrayError::Unsupported("trust set-signed requires --match-prefix PREFIX".into())
    })?;
    set_require_signed_commit(&trust_home()?, &prefix, enabled)?;
    Ok(())
}

fn trust_set_allow_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<()> {
    let mut match_prefix = None;
    let mut allow = true;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--match-prefix" => {
                match_prefix = Some(arguments.next().ok_or_else(|| {
                    PrayError::Unsupported("--match-prefix requires VALUE".into())
                })?);
            }
            "--allow" => {
                let value = arguments
                    .next()
                    .ok_or_else(|| PrayError::Unsupported("--allow requires true|false".into()))?;
                allow = parse_bool_flag(&value)?;
            }
            other => {
                return Err(PrayError::Unsupported(format!(
                    "unknown trust set-allow argument: {other}"
                )))
            }
        }
    }
    let prefix = match_prefix.ok_or_else(|| {
        PrayError::Unsupported("trust set-allow requires --match-prefix PREFIX".into())
    })?;
    set_allow(&trust_home()?, &prefix, allow)?;
    Ok(())
}

fn trust_import_repo_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<()> {
    let source_url = arguments
        .next()
        .ok_or_else(|| PrayError::Unsupported("trust import-repo requires SOURCE_URL".into()))?;
    let clone_url = source_url.strip_prefix("git+").unwrap_or(&source_url);
    let mut match_prefix = None;
    while let Some(argument) = arguments.next() {
        if argument == "--match-prefix" {
            match_prefix =
                Some(arguments.next().ok_or_else(|| {
                    PrayError::Unsupported("--match-prefix requires VALUE".into())
                })?);
        } else {
            return Err(PrayError::Unsupported(format!(
                "unknown trust import-repo argument: {argument}"
            )));
        }
    }
    let project_root = env::current_dir()?;
    let repository = git_source_cache_directory(&project_root, clone_url);
    if !repository.join(".git").is_dir() {
        return Err(PrayError::Resolution(format!(
            "no cached git repository for {clone_url} at {}",
            repository.display()
        )));
    }
    let added = import_signing_keys_from_repository(
        &trust_home()?,
        clone_url,
        &repository,
        match_prefix.as_deref().or(Some(clone_url)),
    )?;
    println!("imported {added} key(s) from {}", repository.display());
    Ok(())
}

fn trust_import_registry_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<()> {
    let source_url = arguments.next().ok_or_else(|| {
        PrayError::Unsupported("trust import-registry requires SOURCE_URL".into())
    })?;
    let mut match_prefix = None;
    let mut include_host_key = pray_core::ssh_client::is_pray_ssh_url(&source_url);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--match-prefix" => {
                match_prefix = Some(arguments.next().ok_or_else(|| {
                    PrayError::Unsupported("--match-prefix requires VALUE".into())
                })?);
            }
            "--host-key" => {
                include_host_key = true;
            }
            "--no-host-key" => {
                include_host_key = false;
            }
            other => {
                return Err(PrayError::Unsupported(format!(
                    "unknown trust import-registry argument: {other}"
                )))
            }
        }
    }
    let result = import_registry_trust(
        &trust_home()?,
        &source_url,
        match_prefix.as_deref(),
        include_host_key,
    )?;
    println!(
        "imported {} publisher fingerprint(s) and {} host key(s) for {}",
        result.publishers_added,
        result.host_keys_added,
        match_prefix.as_deref().unwrap_or(&source_url)
    );
    Ok(())
}

fn trust_check_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<()> {
    let source = arguments.next();
    if arguments.next().is_some() {
        return Err(PrayError::Unsupported(
            "trust check accepts at most one SOURCE argument".into(),
        ));
    }
    let (source_description, body) = fetch_compromised_feed(source.as_deref())?;
    let entries = parse_compromised_feed(&body, &source_description);
    let hits = check_compromised_keys(&trust_home()?, &entries)?;
    if hits.is_empty() {
        println!(
            "no compromised trusted signing keys detected (checked against {source_description})"
        );
        return Ok(());
    }
    let hit_count = hits.len();
    for (key, scopes, matches) in &hits {
        println!("[compromised] {key}");
        println!(
            "  scopes: {}",
            scopes.iter().cloned().collect::<Vec<_>>().join(", ")
        );
        for entry in matches {
            if let Some(reason) = &entry.reason {
                println!("  reason: {reason}");
            }
            if let Some(reference) = &entry.reference {
                println!("  reference: {reference}");
            }
            if let Some(reported_at) = &entry.reported_at {
                println!("  reported_at: {reported_at}");
            }
        }
    }
    Err(PrayError::Integrity(format!(
        "found {hit_count} compromised trusted key(s) in {source_description}"
    )))
}

fn fetch_compromised_feed(source: Option<&str>) -> PrayResult<(String, String)> {
    let url = source.unwrap_or(DEFAULT_COMPROMISED_KEYS_SOURCE);
    if url.starts_with("http://") || url.starts_with("https://") {
        let mut response = ureq::get(url)
            .header(
                "User-Agent",
                concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")),
            )
            .call()
            .map_err(|error| {
                PrayError::Unsupported(format!("HTTP request failed for {url}: {error}"))
            })?;
        let status = response.status();
        let body = response
            .body_mut()
            .read_to_string()
            .unwrap_or_default();
        if !status.is_success() {
            return Err(PrayError::Unsupported(format!(
                "compromised-key source returned HTTP {}",
                status.as_u16()
            )));
        }
        return Ok((url.to_string(), body));
    }

    let path = PathBuf::from(url);
    let body = fs::read_to_string(&path)?;
    Ok((path.display().to_string(), body))
}

fn parse_bool_flag(value: &str) -> PrayResult<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        other => Err(PrayError::Unsupported(format!(
            "expected true or false, got {other}"
        ))),
    }
}
