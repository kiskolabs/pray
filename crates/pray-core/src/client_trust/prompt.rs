use std::io::{IsTerminal, Write};
use std::path::Path;

use crate::{PrayError, PrayResult};

use super::enforce::{env_truthy, signer_matches_allowed};
use super::git::{repository_signing_keys, trust_git_output};
use super::policy::{best_rule, keys_missing_for_trust_scope, load_policy_or_default};

#[derive(Debug, Clone)]
struct HeadAssessment {
    commit: String,
    short_commit: String,
    author_name: String,
    author_email: String,
    authored_at: String,
    subject: String,
    signature_status: String,
    signature_signer: String,
    signature_key: String,
    signature_fingerprint: String,
}

pub fn prompt_import_signing_keys_for_source(
    home: &Path,
    source_url: &str,
    repository: &Path,
    global_scope: bool,
) -> PrayResult<()> {
    let keys = repository_signing_keys(home, source_url, repository);
    if keys.is_empty() {
        eprintln!(
            "[pray][trust] no signer key/fingerprint found on HEAD (nothing to import): {}",
            repository.display()
        );
        return Ok(());
    }
    let missing = keys_missing_for_trust_scope(home, source_url, &keys, global_scope)?;
    if missing.is_empty() {
        eprintln!("[pray][trust] signer key already trusted for {source_url}");
        return Ok(());
    }

    if !std::io::stdin().is_terminal() || !std::io::stderr().is_terminal() {
        return Err(PrayError::Unsupported(format!(
            "--trust requires an interactive terminal to confirm signer key import for {source_url}"
        )));
    }

    print_head_assessment(home, source_url, repository, global_scope, &missing, true)?;
    let mut stderr = std::io::stderr();
    write!(
        stderr,
        "[pray][trust] add these keys to trust policy? [y/N]: "
    )?;
    stderr.flush()?;

    if !confirmed_yes()? {
        eprintln!("[pray][trust] declined key import for {source_url}");
        return Ok(());
    }

    for key in &missing {
        super::commands::add_allowed_signing_key(
            home,
            key,
            if global_scope { None } else { Some(source_url) },
        )?;
    }
    eprintln!(
        "[pray][trust] imported {} key(s) for {}",
        missing.len(),
        source_url
    );
    Ok(())
}

pub fn prompt_untrusted_source_consent(
    home: &Path,
    source_url: &str,
    repository: &Path,
) -> PrayResult<()> {
    let policy = load_policy_or_default(home)?;
    let rule = best_rule(&policy, source_url);
    if !rule.allow {
        return Ok(());
    }

    let assessment = head_assessment(home, source_url, repository);
    let signature_status = assessment
        .as_ref()
        .map(|value| value.signature_status.as_str())
        .unwrap_or_default();
    let good_signature = signature_status == "G";
    let trusted_signer = signer_matches_allowed(home, source_url, rule, repository);
    if good_signature || trusted_signer {
        return Ok(());
    }

    if env_truthy("PRAY_TRUST_ASSUME_YES") {
        eprintln!(
            "[pray][trust] auto-consent enabled via PRAY_TRUST_ASSUME_YES for untrusted source {source_url}"
        );
        return Ok(());
    }

    if !std::io::stdin().is_terminal() || !std::io::stderr().is_terminal() {
        return Err(PrayError::Integrity(format!(
            "untrusted source requires interactive consent (no verified signature/trusted signer): {source_url}"
        )));
    }

    eprintln!("[pray][trust] untrusted source assessment");
    print_head_assessment(home, source_url, repository, false, &[], false)?;
    eprintln!(
        "[pray][trust] reason: source has no verified-good signature and signer is not trusted in policy"
    );

    let mut stderr = std::io::stderr();
    write!(
        stderr,
        "[pray][trust] continue with this untrusted source? [y/N]: "
    )?;
    stderr.flush()?;

    if !confirmed_yes()? {
        return Err(PrayError::Integrity(format!(
            "installation aborted by user for untrusted source {source_url}"
        )));
    }
    Ok(())
}

fn print_head_assessment(
    home: &Path,
    source_url: &str,
    repository: &Path,
    global_scope: bool,
    proposed_keys: &[String],
    import_prompt: bool,
) -> PrayResult<()> {
    eprintln!("[pray][trust] source: {source_url}");
    eprintln!("[pray][trust] repo: {}", repository.display());
    eprintln!(
        "[pray][trust] policy scope: {}",
        if global_scope {
            "<default/global>"
        } else {
            "<source-specific>"
        }
    );
    if let Some(assessment) = head_assessment(home, source_url, repository) {
        eprintln!(
            "[pray][trust] head: {} ({})",
            assessment.short_commit,
            if assessment.commit.is_empty() {
                "-"
            } else {
                &assessment.commit
            }
        );
        eprintln!(
            "[pray][trust] author: {} <{}>",
            assessment.author_name, assessment.author_email
        );
        eprintln!("[pray][trust] date: {}", assessment.authored_at);
        eprintln!("[pray][trust] subject: {}", assessment.subject);
        if !assessment.signature_status.is_empty() {
            eprintln!(
                "[pray][trust] signature: {} ({})",
                signature_status_human(&assessment.signature_status),
                assessment.signature_status
            );
        } else if !import_prompt {
            eprintln!("[pray][trust] signature: unknown");
        }
        if !assessment.signature_signer.is_empty() {
            eprintln!("[pray][trust] signer: {}", assessment.signature_signer);
        }
        if !assessment.signature_key.is_empty() {
            eprintln!("[pray][trust] signer key id: {}", assessment.signature_key);
        }
        if !assessment.signature_fingerprint.is_empty() {
            eprintln!(
                "[pray][trust] signer fingerprint: {}",
                assessment.signature_fingerprint
            );
        }
    } else if !import_prompt {
        eprintln!("[pray][trust] unable to read HEAD signing metadata");
    }
    if import_prompt {
        eprintln!("[pray][trust] proposed keys:");
        for key in proposed_keys {
            eprintln!("[pray][trust]   - {key}");
        }
    }
    Ok(())
}

fn head_assessment(home: &Path, source_url: &str, repository: &Path) -> Option<HeadAssessment> {
    let format = "%H%n%h%n%an%n%ae%n%aI%n%s%n%G?%n%GS%n%GK%n%GF";
    let raw = trust_git_output(
        home,
        source_url,
        repository,
        &["log", "-1", &format!("--format={format}")],
    )
    .ok()?;
    let mut lines = raw.lines();
    Some(HeadAssessment {
        commit: lines.next().unwrap_or_default().to_string(),
        short_commit: lines.next().unwrap_or_default().to_string(),
        author_name: lines.next().unwrap_or_default().to_string(),
        author_email: lines.next().unwrap_or_default().to_string(),
        authored_at: lines.next().unwrap_or_default().to_string(),
        subject: lines.next().unwrap_or_default().to_string(),
        signature_status: lines.next().unwrap_or_default().to_string(),
        signature_signer: lines.next().unwrap_or_default().to_string(),
        signature_key: lines.next().unwrap_or_default().to_string(),
        signature_fingerprint: lines.next().unwrap_or_default().to_string(),
    })
}

fn signature_status_human(code: &str) -> &'static str {
    match code {
        "G" => "good signature",
        "U" => "good signature (untrusted key)",
        "B" => "bad signature",
        "N" => "no signature",
        "E" => "signature verification error",
        _ => "unknown signature state",
    }
}

fn confirmed_yes() -> PrayResult<bool> {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(matches!(
        input.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}
