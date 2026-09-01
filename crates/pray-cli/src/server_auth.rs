use crate::server::{response_with_status, Response};
use crate::server_auth_delivery::deliver_verification_code;
use pray_core::auth::{
    bearer_token_from_authorization, AuthPasskeyChallengeRequest, AuthPasskeyChallengeResponse,
    AuthPasskeyEnrollmentRequest, AuthPasskeyEnrollmentResponse, AuthPasskeyLoginRequest,
    AuthPasskeyLoginResponse, AuthRegistrationRequest, AuthRegistrationResponse,
    AuthSessionRequest, AuthSessionResponse, AuthSshKeyChallengeRequest,
    AuthSshKeyChallengeResponse, AuthSshKeyEnrollmentRequest, AuthSshKeyEnrollmentResponse,
    AuthSshKeyLoginRequest, AuthSshKeyLoginResponse, AuthVerificationRequest,
    AuthVerificationResponse, RegistryAuthStore,
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
    if let Some(code) = response.verification_code.as_deref() {
        deliver_verification_code(root, &response.email, code)?;
    }
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

pub(crate) fn auth_passkey_enroll_response(
    root: &Path,
    authorization: Option<&str>,
    body: &[u8],
) -> PrayResult<Response> {
    if !read_registry_trust_settings(root)?.passkeys_enabled {
        return Ok(disabled_auth_response());
    }
    let request: AuthPasskeyEnrollmentRequest =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "auth passkey enrollment",
            message: error.to_string(),
        })?;
    let Some(session) = session_from_authorization(root, authorization)? else {
        return Ok(protected_auth_response());
    };
    if session.email != request.email {
        return Ok(protected_auth_response());
    }
    let store = RegistryAuthStore::open(root)?;
    let response: AuthPasskeyEnrollmentResponse = store.enroll_passkey(
        &request.email,
        &request.credential_id,
        &request.public_key,
        request.label.as_deref(),
    )?;
    json_ok(response)
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
    json_ok(response)
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
    json_ok(response)
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
    json_ok(response)
}

pub(crate) fn auth_ssh_key_enroll_response(
    root: &Path,
    authorization: Option<&str>,
    body: &[u8],
) -> PrayResult<Response> {
    if !read_registry_trust_settings(root)?.ssh_keys_enabled {
        return Ok(disabled_auth_response());
    }
    let request: AuthSshKeyEnrollmentRequest =
        serde_json::from_slice(body).map_err(|error| PrayError::Parse {
            kind: "auth ssh key enrollment",
            message: error.to_string(),
        })?;
    let Some(session) = session_from_authorization(root, authorization)? else {
        return Ok(protected_auth_response());
    };
    if session.email != request.email {
        return Ok(protected_auth_response());
    }
    let store = RegistryAuthStore::open(root)?;
    let response: AuthSshKeyEnrollmentResponse = store.enroll_ssh_key(
        &request.email,
        &request.public_key,
        request.label.as_deref(),
    )?;
    json_ok(response)
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
    json_ok(response)
}

fn session_from_authorization(
    root: &Path,
    authorization: Option<&str>,
) -> PrayResult<Option<AuthSessionResponse>> {
    let Some(token) = bearer_token_from_authorization(authorization) else {
        return Ok(None);
    };
    RegistryAuthStore::open(root)?.resolve_session(&token)
}

fn json_ok(value: impl serde::Serialize) -> PrayResult<Response> {
    let body =
        serde_json::to_vec(&value).map_err(|error| PrayError::Manifest(error.to_string()))?;
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
#[path = "server_auth_tests.rs"]
mod server_auth_handler_tests;
