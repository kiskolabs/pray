use crate::server::read_registry_index;
use crate::server_http::Response;
use pray_core::registry::{ConfessionSubmission, RegistryPackageMetadata};
use pray_core::trust::read_registry_trust_settings;
use pray_core::{PrayError, PrayResult};
use std::fs;
use std::path::Path;

pub(crate) fn html_root_response(root: &Path) -> PrayResult<Response> {
    let index = read_registry_index(root)?;
    let trust = read_registry_trust_settings(root)?;
    let mut list_items = String::new();
    for package in index.packages {
        list_items.push_str(&format!(
            "<li><a href=\"/packages/{path}\">{package}</a></li>",
            path = html_escape(&package)
        ));
    }
    let body = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Pray distribution point</title></head><body><h1>Pray distribution point</h1><p>Spec: {spec}</p><p>Email confirmation: {email}</p><p>Passkeys: {passkeys}</p><p>SSH keys: {ssh_keys}</p><p>SSH-agent signing: {ssh_agent}</p><ul>{packages}</ul></body></html>",
        spec = html_escape(&index.spec),
        email = trust.email_confirmation_label(),
        passkeys = trust.passkeys_label(),
        ssh_keys = trust.ssh_keys_label(),
        ssh_agent = trust.ssh_agent_label(),
        packages = list_items,
    );
    Ok(Response {
        status: 200,
        content_type: "text/html; charset=utf-8".to_string(),
        body: body.into_bytes(),
    })
}

pub(crate) fn html_package_response(root: &Path, path: &str) -> PrayResult<Response> {
    let package_name = path.trim_start_matches("/packages/");
    let metadata_path = root
        .join("v1/packages")
        .join(package_name)
        .with_extension("json");
    let metadata_text = fs::read_to_string(&metadata_path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            PrayError::Resolution(format!("package metadata not found: {}", package_name))
        } else {
            PrayError::from(error)
        }
    })?;
    let metadata: RegistryPackageMetadata =
        serde_json::from_str(&metadata_text).map_err(|error| PrayError::Parse {
            kind: "registry metadata",
            message: error.to_string(),
        })?;
    let confessions = read_confessions(root, package_name)?;
    let body = render_package_page(package_name, &metadata, &confessions);
    Ok(Response {
        status: 200,
        content_type: "text/html; charset=utf-8".to_string(),
        body: body.into_bytes(),
    })
}

fn render_package_page(
    package_name: &str,
    metadata: &RegistryPackageMetadata,
    confessions: &[ConfessionSubmission],
) -> String {
    let mut versions = String::new();
    for version in &metadata.versions {
        let mut details = String::new();
        if let Some(signer) = version.signer.as_ref() {
            details.push_str(&format!("<div>Signer: {}</div>", html_escape(signer)));
        }
        if let Some(signature) = version.signature.as_ref() {
            details.push_str(&format!("<div>Signature: {}</div>", html_escape(signature)));
        }
        if let Some(published_at) = version.published_at.as_ref() {
            details.push_str(&format!(
                "<div>Published at: {}</div>",
                html_escape(published_at)
            ));
        }
        versions.push_str(&format!(
            "<li><a href=\"/{artifact}\">{version}</a>{details}</li>",
            artifact = html_escape(&version.artifact),
            version = html_escape(&version.version),
            details = details,
        ));
    }
    let accepted = confessions
        .iter()
        .filter(|entry| entry.status == "accepted")
        .count();
    let rejected = confessions
        .iter()
        .filter(|entry| entry.status == "rejected")
        .count();
    let mut confession_items = String::new();
    for confession in confessions {
        confession_items.push_str(&format!(
            "<li><strong>{}</strong> {}{}</li>",
            html_escape(confession.status.as_str()),
            html_escape(confession.version.as_str()),
            confession
                .note
                .as_ref()
                .map(|note| format!(": {}", html_escape(note)))
                .unwrap_or_default()
        ));
    }
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{package}</title></head><body><h1>{package}</h1><p>Accepted: {accepted}</p><p>Rejected: {rejected}</p><h2>Versions</h2><ul>{versions}</ul><h2>Confessions</h2><ul>{confession_items}</ul></body></html>",
        package = html_escape(package_name),
        accepted = accepted,
        rejected = rejected,
        versions = versions,
        confession_items = confession_items,
    )
}

fn read_confessions(root: &Path, package_name: &str) -> PrayResult<Vec<ConfessionSubmission>> {
    let path = root.join("v1/confessions.jsonl");
    let Ok(text) = fs::read_to_string(path) else {
        return Ok(Vec::new());
    };
    let mut confessions = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let confession: ConfessionSubmission =
            serde_json::from_str(line).map_err(|error| PrayError::Parse {
                kind: "confession",
                message: error.to_string(),
            })?;
        if confession.package == package_name {
            confessions.push(confession);
        }
    }
    Ok(confessions)
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
