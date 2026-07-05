use crate::{PrayError, PrayResult};
use std::fs;
use std::path::{Path, PathBuf};

pub fn persistent_pray_home() -> PrayResult<PathBuf> {
    if let Ok(path) = std::env::var("PRAY_USER_HOME") {
        if !path.trim().is_empty() {
            return Ok(PathBuf::from(path));
        }
    }
    let home = std::env::var_os("HOME").map(PathBuf::from).ok_or_else(|| {
        PrayError::Unsupported("cannot determine home directory from HOME".into())
    })?;
    Ok(home.join(".pray"))
}

pub fn effective_trust_home() -> PrayResult<PathBuf> {
    if let Ok(path) = std::env::var("PRAY_HOME") {
        if !path.trim().is_empty() {
            return Ok(PathBuf::from(path));
        }
    }
    persistent_pray_home()
}

pub fn prepare_ephemeral_home() -> PrayResult<PathBuf> {
    let persistent = persistent_pray_home()?;
    let temp_root = std::env::temp_dir().join(format!("pray-ephemeral-{}", std::process::id()));
    fs::create_dir_all(&temp_root)?;
    copy_trust_state(&persistent, &temp_root)?;
    std::env::set_var("PRAY_HOME", temp_root.as_os_str());
    Ok(temp_root)
}

pub fn copy_trust_state(from_home: &Path, to_home: &Path) -> PrayResult<()> {
    let source_policy = from_home.join("trust.toml");
    if source_policy.is_file() {
        let destination = to_home.join("trust.toml");
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&source_policy, &destination)?;
    }

    let source_trust_dir = from_home.join("trust");
    if source_trust_dir.is_dir() {
        copy_dir_recursive(&source_trust_dir, &to_home.join("trust"))?;
    }
    Ok(())
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> PrayResult<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ephemeral_home_copies_trust_policy() {
        let base = std::env::temp_dir().join(format!("pray-home-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join("trust/allowed_signers")).expect("dirs");
        fs::write(base.join("trust.toml"), "[default]\nallow = true\n").expect("policy");
        fs::write(
            base.join("trust/allowed_signers/example.signers"),
            "signer\n",
        )
        .expect("signers");

        let ephemeral = base.join("ephemeral");
        copy_trust_state(&base, &ephemeral).expect("copy");

        assert!(ephemeral.join("trust.toml").is_file());
        assert!(ephemeral
            .join("trust/allowed_signers/example.signers")
            .is_file());

        let _ = fs::remove_dir_all(&base);
    }
}
