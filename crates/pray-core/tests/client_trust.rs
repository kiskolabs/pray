use pray_core::client_trust::{
    add_allowed_signing_key, list_policy, load_policy, save_policy, show_policy_toml,
    ClientTrustPolicy, ClientTrustRule, TrustListScope,
};
use std::fs;
use std::path::PathBuf;

#[test]
fn trust_show_and_list_round_trip_policy() {
    let home = temp_trust_home();
    add_allowed_signing_key(&home, "sha256:example", Some("https://github.com/example/"))
        .expect("add key");

    let shown = show_policy_toml(&home).expect("show");
    assert!(shown.contains("SHA256:EXAMPLE"));

    let listed = list_policy(
        &home,
        TrustListScope::All,
        Some("https://github.com/example/repo.git"),
    )
    .expect("list");
    assert!(listed.contains("scope: local"));
    assert!(listed.contains("SHA256:EXAMPLE"));

    let _ = fs::remove_dir_all(&home);
}

#[test]
fn trust_policy_persists_via_toml_round_trip() {
    let home = temp_trust_home();
    let policy = ClientTrustPolicy {
        default: ClientTrustRule {
            require_signed_commit: true,
            ..ClientTrustRule::default()
        },
        rules: Vec::new(),
    };
    save_policy(&home, &policy).expect("save");
    let loaded = load_policy(&home).expect("load").expect("policy file");
    assert!(loaded.default.require_signed_commit);

    let _ = fs::remove_dir_all(&home);
}

fn temp_trust_home() -> PathBuf {
    let home = std::env::temp_dir().join(format!(
        "pray-trust-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    fs::create_dir_all(&home).expect("home");
    home
}
