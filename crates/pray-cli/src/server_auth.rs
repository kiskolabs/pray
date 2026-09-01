use crate::server::{response_with_status, Response};
use pray_core::auth::{
    AuthPasskeyChallengeRequest, AuthPasskeyChallengeResponse, AuthPasskeyEnrollmentRequest,
    AuthPasskeyLoginRequest, AuthPasskeyLoginResponse, AuthRegistrationRequest,
    AuthRegistrationResponse, AuthSessionRequest, AuthSshKeyChallengeRequest,
    AuthSshKeyChallengeResponse, AuthSshKeyEnrollmentRequest, AuthSshKeyLoginRequest,
    AuthSshKeyLoginResponse, AuthVerificationRequest, AuthVerificationResponse, RegistryAuthStore,
};
use pray_core::trust::read_registry_trust_settings;
use pray_core::{PrayError, PrayResult};
use std::path::Path;

pub(crate) fn auth_register_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    let request: AuthRegistrationRequest =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "auth registration",
            message: error.to_string(),
        })?;
    let trust = read_registry_trust_settings(root)?;
    let store = RegistryAuthStore::open(root)?;
    let response: AuthRegistrationResponse =
        store.register_email(&request.email, trust.email_confirmation)?;
    let body = serde_json::to_vec(&serde_json::json!({
        "email": response.email,
        "verified": response.verified,
    }))
    .map_err(|error| PrayError::Manifest(error.to_string()))?;
    Ok(Response {
        status: 201,
        content_type: "application/json".to_string(),
        body,
    })
}

pub(crate) fn auth_verify_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    let request: AuthVerificationRequest =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "auth verification",
            message: error.to_string(),
        })?;
    let store = RegistryAuthStore::open(root)?;
    let response: AuthVerificationResponse = store.verify_email(&request.email, &request.code)?;
    let body =
        serde_json::to_vec(&response).map_err(|error| PrayError::Manifest(error.to_string()))?;
    Ok(Response {
        status: 200,
        content_type: "application/json".to_string(),
        body,
    })
}

pub(crate) fn auth_session_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    let request: AuthSessionRequest =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "auth session",
            message: error.to_string(),
        })?;
    let _ = (root, request);
    Ok(protected_auth_response())
}

pub(crate) fn auth_passkey_enroll_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    let request: AuthPasskeyEnrollmentRequest =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "auth passkey enrollment",
            message: error.to_string(),
        })?;
    let _ = (root, request);
    Ok(protected_auth_response())
}

pub(crate) fn auth_passkey_challenge_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    if !read_registry_trust_settings(root)?.passkeys_enabled {
        return Ok(disabled_auth_response());
    }
    let request: AuthPasskeyChallengeRequest =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "auth passkey challenge",
            message: error.to_string(),
        })?;
    let store = RegistryAuthStore::open(root)?;
    let response: AuthPasskeyChallengeResponse =
        store.request_passkey_challenge(&request.credential_id)?;
    let body =
        serde_json::to_vec(&response).map_err(|error| PrayError::Manifest(error.to_string()))?;
    Ok(Response {
        status: 200,
        content_type: "application/json".to_string(),
        body,
    })
}

pub(crate) fn auth_passkey_login_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    if !read_registry_trust_settings(root)?.passkeys_enabled {
        return Ok(disabled_auth_response());
    }
    let request: AuthPasskeyLoginRequest =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "auth passkey login",
            message: error.to_string(),
        })?;
    let store = RegistryAuthStore::open(root)?;
    let response: AuthPasskeyLoginResponse = store.respond_passkey_challenge(
        &request.credential_id,
        &request.challenge_id,
        &request.signature,
    )?;
    let body =
        serde_json::to_vec(&response).map_err(|error| PrayError::Manifest(error.to_string()))?;
    Ok(Response {
        status: 200,
        content_type: "application/json".to_string(),
        body,
    })
}

pub(crate) fn auth_ssh_key_challenge_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    if !read_registry_trust_settings(root)?.ssh_keys_enabled {
        return Ok(disabled_auth_response());
    }
    let request: AuthSshKeyChallengeRequest =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "auth ssh key challenge",
            message: error.to_string(),
        })?;
    let store = RegistryAuthStore::open(root)?;
    let response: AuthSshKeyChallengeResponse =
        store.request_ssh_key_challenge(&request.public_key)?;
    let body =
        serde_json::to_vec(&response).map_err(|error| PrayError::Manifest(error.to_string()))?;
    Ok(Response {
        status: 200,
        content_type: "application/json".to_string(),
        body,
    })
}

pub(crate) fn auth_ssh_key_enroll_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    let request: AuthSshKeyEnrollmentRequest =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "auth ssh key enrollment",
            message: error.to_string(),
        })?;
    let _ = (root, request);
    Ok(protected_auth_response())
}

pub(crate) fn auth_ssh_key_login_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    if !read_registry_trust_settings(root)?.ssh_keys_enabled {
        return Ok(disabled_auth_response());
    }
    let request: AuthSshKeyLoginRequest =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "auth ssh key login",
            message: error.to_string(),
        })?;
    let store = RegistryAuthStore::open(root)?;
    let response: AuthSshKeyLoginResponse = store.respond_ssh_key_challenge(
        &request.public_key,
        &request.challenge_id,
        &request.signature,
    )?;
    let body =
        serde_json::to_vec(&response).map_err(|error| PrayError::Manifest(error.to_string()))?;
    Ok(Response {
        status: 200,
        content_type: "application/json".to_string(),
        body,
    })
}

fn protected_auth_response() -> Response {
    response_with_status(
        403,
        "text/plain",
        b"authentication proof is required for this operation".to_vec(),
    )
}

fn disabled_auth_response() -> Response {
    response_with_status(
        403,
        "text/plain",
        b"this authentication method is disabled".to_vec(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pray_core::auth::RegistryAuthStore;
    use pray_core::trust::EmailConfirmationPolicy;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn registration_does_not_return_verification_secret() {
        let root = temporary_root("register");
        let response = auth_register_response(&root, br#"{"email":"alice@example.com"}"#)
            .expect("registration response");
        let body: serde_json::Value = serde_json::from_slice(&response.body).expect("json");

        assert_eq!(response.status, 201);
        assert_eq!(body.get("verification_code"), None);
    }

    #[test]
    fn email_session_and_public_key_enrollment_require_authenticated_workflow() {
        let root = temporary_root("protected");
        let store = RegistryAuthStore::open(&root).expect("auth store");
        store
            .register_email("alice@example.com", EmailConfirmationPolicy::Disabled)
            .expect("registration");
        let public_key = ssh_public_key();

        let session = auth_session_response(&root, br#"{"email":"alice@example.com"}"#)
            .expect("session response");
        let passkey = auth_passkey_enroll_response(
            &root,
            serde_json::to_vec(&json!({
                "email": "alice@example.com",
                "credential_id": "credential-1",
                "public_key": public_key,
            }))
            .expect("passkey body")
            .as_slice(),
        )
        .expect("passkey response");
        let ssh_key = auth_ssh_key_enroll_response(
            &root,
            serde_json::to_vec(&json!({
                "email": "alice@example.com",
                "public_key": public_key,
            }))
            .expect("ssh body")
            .as_slice(),
        )
        .expect("ssh response");

        assert_eq!(session.status, 403);
        assert_eq!(passkey.status, 403);
        assert_eq!(ssh_key.status, 403);
    }

    #[test]
    fn disabled_key_methods_reject_login_challenges() {
        let root = temporary_root("disabled");
        let store = RegistryAuthStore::open(&root).expect("auth store");
        store
            .register_email("alice@example.com", EmailConfirmationPolicy::Disabled)
            .expect("registration");
        store
            .enroll_passkey("alice@example.com", "credential-1", &ssh_public_key(), None)
            .expect("passkey");

        let response =
            auth_passkey_challenge_response(&root, br#"{"credential_id":"credential-1"}"#)
                .expect("challenge response");

        assert_eq!(response.status, 403);
    }

    fn temporary_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "pray-server-auth-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::create_dir_all(root.join("v1")).expect("root");
        root
    }

    fn ssh_public_key() -> String {
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcH"
            .to_string()
    }
}
