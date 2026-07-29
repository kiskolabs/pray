use crate::cli_parse::parse_command;
use crate::registry_ops::current_signer;
use pray_core::PrayError;

#[test]
fn omits_the_serve_command() {
    assert!(matches!(
        parse_command(vec!["serve".to_string()]),
        Err(PrayError::Usage(message)) if message == "unknown command: serve"
    ));
}

#[test]
fn rejects_session_tokens_without_auth_storage() {
    std::env::set_var("PRAY_SESSION_TOKEN", "session-token");

    let error = current_signer().expect_err("session tokens require auth storage");

    std::env::remove_var("PRAY_SESSION_TOKEN");
    assert!(matches!(
        error,
        PrayError::Unsupported(message) if message == "this build was compiled without auth support"
    ));
}
