use std::fs;

pub fn write_auth_registry_fixture(root: &std::path::Path) {
    fs::create_dir_all(root.join("v1")).expect("auth root directories");
    fs::write(
        root.join("v1/index.json"),
        r#"{
            "spec": "prayfile-distribution-1",
            "packages": []
        }"#,
    )
    .expect("write auth index");
    fs::write(
        root.join("v1/trust.json"),
        r#"{
            "email_confirmation": "required",
            "passkeys_enabled": true,
            "ssh_keys_enabled": true,
            "ssh_agent_signing_enabled": true
        }"#,
    )
    .expect("write auth trust settings");
}
