use crate::server::Response;
use pray_core::auth::{
    AuthPasskeyChallengeRequest, AuthPasskeyChallengeResponse, AuthPasskeyEnrollmentRequest,
    AuthPasskeyEnrollmentResponse, AuthPasskeyLoginRequest, AuthPasskeyLoginResponse,
    AuthRegistrationRequest, AuthRegistrationResponse, AuthSessionKind, AuthSessionRequest,
    AuthSessionResponse, AuthSshKeyChallengeRequest, AuthSshKeyChallengeResponse,
    AuthSshKeyEnrollmentRequest, AuthSshKeyEnrollmentResponse, AuthSshKeyLoginRequest,
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
    let body =
        serde_json::to_vec(&response).map_err(|error| PrayError::Manifest(error.to_string()))?;
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
    let store = RegistryAuthStore::open(root)?;
    let response: AuthSessionResponse =
        store.issue_session(&request.email, AuthSessionKind::Email)?;
    let body =
        serde_json::to_vec(&response).map_err(|error| PrayError::Manifest(error.to_string()))?;
    Ok(Response {
        status: 200,
        content_type: "application/json".to_string(),
        body,
    })
}

pub(crate) fn auth_passkey_enroll_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
    let request: AuthPasskeyEnrollmentRequest =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "auth passkey enrollment",
            message: error.to_string(),
        })?;
    let store = RegistryAuthStore::open(root)?;
    let response: AuthPasskeyEnrollmentResponse = store.enroll_passkey(
        &request.email,
        &request.credential_id,
        &request.public_key,
        request.label.as_deref(),
    )?;
    let body =
        serde_json::to_vec(&response).map_err(|error| PrayError::Manifest(error.to_string()))?;
    Ok(Response {
        status: 201,
        content_type: "application/json".to_string(),
        body,
    })
}

pub(crate) fn auth_passkey_challenge_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
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
    let store = RegistryAuthStore::open(root)?;
    let response: AuthSshKeyEnrollmentResponse = store.enroll_ssh_key(
        &request.email,
        &request.public_key,
        request.label.as_deref(),
    )?;
    let body =
        serde_json::to_vec(&response).map_err(|error| PrayError::Manifest(error.to_string()))?;
    Ok(Response {
        status: 201,
        content_type: "application/json".to_string(),
        body,
    })
}

pub(crate) fn auth_ssh_key_login_response(root: &Path, body: &[u8]) -> PrayResult<Response> {
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
