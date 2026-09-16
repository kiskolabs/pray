use pray_core::distribution::{
    parse_registry_distribution_settings, read_registry_distribution_settings,
};
use std::path::Path;

#[test]
fn missing_file_disables_extra_protocols() {
    let settings = read_registry_distribution_settings(Path::new("/no/such/root"))
        .expect("missing file is empty policy");
    assert!(!settings.allows_torrent());
    assert!(settings.bootstrap_trackers.is_empty());
    assert!(!settings.enable_dht);
}

#[test]
fn empty_protocols_disable_torrent() {
    let settings = parse_registry_distribution_settings(
        r#"{"spec":"pray-distribution-config-1","protocols":[]}"#,
    )
    .expect("parse");
    assert!(!settings.allows_torrent());
}

#[test]
fn torrent_protocol_opt_in() {
    let settings = parse_registry_distribution_settings(
        r#"{"spec":"pray-distribution-config-1","protocols":["torrent"]}"#,
    )
    .expect("parse");
    assert!(settings.allows_torrent());
}

#[test]
fn unknown_protocol_fails() {
    let error = parse_registry_distribution_settings(
        r#"{"spec":"pray-distribution-config-1","protocols":["ipfs"]}"#,
    )
    .expect_err("unknown protocol");
    assert!(error.to_string().contains("unsupported protocol: ipfs"));
}

#[test]
fn leftover_sidecars_field_fails() {
    let error = parse_registry_distribution_settings(
        r#"{"spec":"pray-distribution-config-1","sidecars":["torrent"]}"#,
    )
    .expect_err("legacy field");
    assert!(error.to_string().contains("sidecars"));
}

#[test]
fn dht_flag_fails() {
    let error = parse_registry_distribution_settings(
        r#"{"spec":"pray-distribution-config-1","protocols":["torrent"],"enable_dht":true}"#,
    )
    .expect_err("dht");
    assert!(error
        .to_string()
        .contains("DHT announce is not implemented"));
}

#[test]
fn trackers_require_torrent_protocol() {
    let error = parse_registry_distribution_settings(
        r#"{"spec":"pray-distribution-config-1","bootstrap_trackers":["http://tracker.example"]}"#,
    )
    .expect_err("trackers");
    assert!(error.to_string().contains("bootstrap_trackers"));
}
