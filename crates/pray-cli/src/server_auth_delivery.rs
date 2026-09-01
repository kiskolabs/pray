use pray_core::{PrayError, PrayResult};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn deliver_verification_code(root: &Path, email: &str, code: &str) -> PrayResult<()> {
    let directory = root.join(".pray");
    fs::create_dir_all(&directory)?;
    let path = directory.join("verification-deliveries.jsonl");
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| PrayError::Resolution(error.to_string()))?
        .as_secs();
    let line = serde_json::json!({
        "email": email,
        "code": code,
        "created_at": created_at,
    });
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    writeln!(file, "{line}").map_err(|error| PrayError::Resolution(error.to_string()))?;
    file.sync_all()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    }
    eprintln!("verification delivered for {email}");
    Ok(())
}
