mod commands;
mod enforce;
mod feed;
mod git;
mod home;
mod import_registry;
mod policy;
mod prompt;
mod ssh_host;

pub use commands::{
    add_allowed_signing_key, ensure_policy_file, import_signing_keys_from_repository, list_policy,
    remove_allowed_signing_key, set_allow, set_require_signed_commit, show_policy_toml,
    TrustListScope,
};
pub use enforce::{enforce_source_trust, env_truthy, gate_git_source, signer_matches_allowed};
pub use feed::{
    check_compromised_keys, parse_compromised_feed, trusted_keys_by_scope, CompromisedKeyEntry,
    DEFAULT_COMPROMISED_KEYS_SOURCE,
};
pub use git::{is_remote_git_url, repository_signing_keys, trust_git_env};
pub use home::{
    copy_trust_state, effective_trust_home, persistent_pray_home, prepare_ephemeral_home,
};
pub use import_registry::{fetch_ssh_publishers, import_registry_trust, ImportRegistryResult};
pub use policy::{
    best_rule, load_policy, load_policy_or_default, normalize_key, save_policy, ClientTrustPolicy,
    ClientTrustRule,
};
pub use prompt::{prompt_import_signing_keys_for_source, prompt_untrusted_source_consent};
pub use ssh_host::{gate_pray_ssh_host, gate_pray_ssh_publisher};
