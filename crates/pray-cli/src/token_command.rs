use pray_core::auth::{RegistryAuthStore, PUBLISH_SCOPE};
use pray_core::{PrayError, PrayResult};
use std::path::PathBuf;

pub(crate) fn run_token_command(arguments: Vec<String>) -> PrayResult<()> {
    let mut iter = arguments.into_iter();
    let subcommand = iter
        .next()
        .ok_or_else(|| PrayError::Unsupported("token requires a subcommand".to_string()))?;
    match subcommand.as_str() {
        "create" => token_create_command(iter),
        "revoke" => token_revoke_command(iter),
        other => Err(PrayError::Unsupported(format!(
            "unknown token command: {other}"
        ))),
    }
}

fn token_create_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<()> {
    let mut root = None;
    let mut email = None;
    let mut scopes = Vec::new();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--root" => {
                root = Some(PathBuf::from(arguments.next().ok_or_else(|| {
                    PrayError::Unsupported("token create requires a path after --root".into())
                })?));
            }
            "--email" => {
                email = Some(arguments.next().ok_or_else(|| {
                    PrayError::Unsupported("token create requires a value after --email".into())
                })?);
            }
            "--scope" => {
                scopes.push(arguments.next().ok_or_else(|| {
                    PrayError::Unsupported("token create requires a value after --scope".into())
                })?);
            }
            other => {
                return Err(PrayError::Unsupported(format!(
                    "unknown token create flag: {other}"
                )))
            }
        }
    }
    let root = root
        .ok_or_else(|| PrayError::Unsupported("token create requires --root PATH".to_string()))?;
    let email = email
        .ok_or_else(|| PrayError::Unsupported("token create requires --email EMAIL".to_string()))?;
    if scopes.is_empty() {
        scopes.push(PUBLISH_SCOPE.to_string());
    }
    let store = RegistryAuthStore::open(&root)?;
    let record = store.issue_publish_token(&email, &scopes)?;
    println!("{}", record.token);
    eprintln!(
        "created publish token for {} with scopes {}",
        record.email,
        record.scopes.join(",")
    );
    eprintln!("set PRAY_PUBLISH_TOKEN to this value for pray publish --server");
    Ok(())
}

fn token_revoke_command(mut arguments: std::vec::IntoIter<String>) -> PrayResult<()> {
    let mut root = None;
    let mut token = None;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--root" => {
                root = Some(PathBuf::from(arguments.next().ok_or_else(|| {
                    PrayError::Unsupported("token revoke requires a path after --root".into())
                })?));
            }
            other if !other.starts_with('-') => token = Some(other.to_string()),
            other => {
                return Err(PrayError::Unsupported(format!(
                    "unknown token revoke flag: {other}"
                )))
            }
        }
    }
    let root = root
        .ok_or_else(|| PrayError::Unsupported("token revoke requires --root PATH".to_string()))?;
    let token =
        token.ok_or_else(|| PrayError::Unsupported("token revoke requires TOKEN".to_string()))?;
    let store = RegistryAuthStore::open(&root)?;
    store.revoke_publish_token(&token)?;
    println!("revoked publish token");
    Ok(())
}
