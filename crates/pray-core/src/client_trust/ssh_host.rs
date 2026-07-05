use crate::client_trust::policy::{best_rule, load_policy};
use crate::ssh_identity::{fingerprint_matches_allowed, normalize_identity};
use crate::{PrayError, PrayResult};
use std::path::Path;
use std::process::Command;

pub fn gate_pray_ssh_host(
    home: &Path,
    source_url: &str,
    host: &str,
    port: u16,
) -> PrayResult<String> {
    if host == "stdio-host" {
        return Ok(String::new());
    }

    let fingerprints = fetch_host_key_fingerprints(host, port)?;
    if fingerprints.is_empty() {
        return Err(PrayError::Integrity(format!(
            "no host keys returned for pray ssh source {source_url}"
        )));
    }

    let Some(policy) = load_policy(home)? else {
        return Ok(fingerprints[0].clone());
    };
    let rule = best_rule(&policy, source_url);
    if rule.allowed_host_keys.is_empty() {
        return Ok(fingerprints[0].clone());
    }

    let matched = fingerprints
        .iter()
        .find(|fingerprint| fingerprint_matches_allowed(fingerprint, &rule.allowed_host_keys));
    let Some(fingerprint) = matched else {
        return Err(PrayError::Integrity(format!(
            "host key for {host} is not listed in allowed_host_keys for {source_url}"
        )));
    };
    Ok(fingerprint.clone())
}

pub fn gate_pray_ssh_publisher(
    home: &Path,
    source_url: &str,
    publisher_fingerprint: &str,
) -> PrayResult<()> {
    let Some(policy) = load_policy(home)? else {
        return Ok(());
    };
    let rule = best_rule(&policy, source_url);
    if rule.allowed_publishers.is_empty() {
        return Ok(());
    }
    if fingerprint_matches_allowed(publisher_fingerprint, &rule.allowed_publishers) {
        Ok(())
    } else {
        Err(PrayError::Integrity(format!(
            "publisher fingerprint {publisher_fingerprint} is not allowed for {source_url}"
        )))
    }
}

pub fn fetch_host_key_fingerprints(host: &str, port: u16) -> PrayResult<Vec<String>> {
    let scan = Command::new("ssh-keyscan")
        .args(["-p", &port.to_string(), "-t", "ed25519,rsa", host])
        .output()
        .map_err(|error| PrayError::Unsupported(format!("failed to run ssh-keyscan: {error}")))?;
    if !scan.status.success() && scan.stdout.is_empty() {
        let stderr = String::from_utf8_lossy(&scan.stderr).trim().to_string();
        return Err(PrayError::Integrity(format!(
            "ssh-keyscan failed for {host}:{port}: {stderr}"
        )));
    }

    let mut listing = Command::new("ssh-keygen")
        .args(["-lf", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|error| PrayError::Unsupported(format!("failed to run ssh-keygen: {error}")))?;
    {
        use std::io::Write;
        let stdin = listing
            .stdin
            .as_mut()
            .ok_or_else(|| PrayError::Unsupported("ssh-keygen stdin unavailable".into()))?;
        stdin.write_all(&scan.stdout).map_err(PrayError::Io)?;
    }
    let output = listing.wait_with_output().map_err(|error| {
        PrayError::Unsupported(format!("failed to read ssh-keygen output: {error}"))
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(PrayError::Integrity(format!(
            "ssh-keygen -lf failed for {host}:{port}: {stderr}"
        )));
    }

    let mut fingerprints = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let Some(fingerprint) = line.split_whitespace().nth(1) else {
            continue;
        };
        let normalized = normalize_identity(fingerprint);
        if !normalized.is_empty() {
            fingerprints.push(normalized);
        }
    }
    Ok(fingerprints)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ssh_keygen_listing_lines() {
        let stdout = b"256 SHA256:abc123 host.example.com (ED25519)\n";
        let mut fingerprints = Vec::new();
        for line in std::str::from_utf8(stdout).expect("utf8").lines() {
            if let Some(fingerprint) = line.split_whitespace().nth(1) {
                fingerprints.push(normalize_identity(fingerprint));
            }
        }
        assert_eq!(fingerprints, vec!["SHA256:ABC123"]);
    }
}
