use crate::ssh_publishers::authorize_ssh_push;
use crate::{PrayError, PrayResult};
use std::path::Path;

pub fn authorize_distribution_push(
    root: &Path,
    bind_host: &str,
    allow_open_push: bool,
    stdio_mode: bool,
) -> PrayResult<()> {
    let stdio_mode = stdio_mode || std::env::var_os("PRAY_SERVE_STDIO").is_some();
    if stdio_mode {
        return authorize_ssh_push(root);
    }

    match authorize_ssh_push(root) {
        Ok(()) => {
            if publishers_configured(root)? {
                return Ok(());
            }
        }
        Err(error) => return Err(error),
    }

    if allow_open_push || is_loopback_bind_host(bind_host) {
        return Ok(());
    }

    Err(PrayError::Resolution(
        "HTTP push requires configured ssh publishers, loopback bind, or --allow-open-push"
            .to_string(),
    ))
}

fn publishers_configured(root: &Path) -> PrayResult<bool> {
    match crate::ssh_publishers::read_ssh_publishers(root)? {
        Some(config) => Ok(!config.publishers.is_empty()),
        None => Ok(false),
    }
}

pub fn is_loopback_bind_host(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "localhost" | "::1" | "0:0:0:0:0:0:0:1")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn loopback_allows_open_push_without_publishers() {
        let root =
            std::env::temp_dir().join(format!("pray-push-auth-loopback-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("temp root");
        authorize_distribution_push(&root, "127.0.0.1", false, false).expect("loopback open push");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn non_loopback_requires_flag_without_publishers() {
        let root =
            std::env::temp_dir().join(format!("pray-push-auth-public-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("temp root");
        let error =
            authorize_distribution_push(&root, "0.0.0.0", false, false).expect_err("public bind");
        assert!(error.to_string().contains("--allow-open-push"));
        authorize_distribution_push(&root, "0.0.0.0", true, false).expect("flag allows");
        let _ = fs::remove_dir_all(&root);
    }
}
