use std::env;
use std::sync::atomic::{AtomicU32, Ordering};

static VERBOSE_LEVEL: AtomicU32 = AtomicU32::new(0);

pub(crate) struct GlobalFlags {
    pub ephemeral: bool,
    pub trust_import: bool,
    pub trust_global: bool,
    pub no_input: bool,
    pub quiet: bool,
    pub verbose: u32,
}

pub(crate) fn extract_global_flags(arguments: Vec<String>) -> (GlobalFlags, Vec<String>) {
    let mut flags = GlobalFlags {
        ephemeral: false,
        trust_import: false,
        trust_global: false,
        no_input: false,
        quiet: false,
        verbose: 0,
    };
    let filtered = arguments
        .into_iter()
        .filter(|argument| match argument.as_str() {
            "--rm" => {
                flags.ephemeral = true;
                false
            }
            "--trust" => {
                flags.trust_import = true;
                false
            }
            "--global" => {
                flags.trust_global = true;
                false
            }
            "--no-input" => {
                flags.no_input = true;
                false
            }
            "--quiet" | "-q" => {
                flags.quiet = true;
                false
            }
            "--verbose" | "-v" => {
                flags.verbose += 1;
                false
            }
            _ => true,
        })
        .collect();
    (flags, filtered)
}

pub(crate) fn configure(quiet: bool, verbose: u32) {
    if quiet {
        env::set_var("PRAY_QUIET", "1");
    }
    VERBOSE_LEVEL.store(verbose, Ordering::Relaxed);
    if verbose > 0 {
        env::set_var("PRAY_VERBOSE", verbose.to_string());
    }
}

pub(crate) fn quiet() -> bool {
    env::var("PRAY_QUIET").ok().is_some_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

pub(crate) fn verbose() -> bool {
    VERBOSE_LEVEL.load(Ordering::Relaxed) > 0
        || env::var("PRAY_VERBOSE")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .is_some_and(|level| level > 0)
}

pub(crate) fn println_stdout(message: impl AsRef<str>) {
    if quiet() {
        return;
    }
    println!("{}", message.as_ref());
}

pub(crate) fn println_verbose(message: impl AsRef<str>) {
    if quiet() || !verbose() {
        return;
    }
    println!("{}", message.as_ref());
}

pub(crate) fn redact_destination_target(target: &str) -> String {
    let Some(scheme_end) = target.find("://") else {
        return target.to_string();
    };
    let after_scheme = &target[scheme_end + 3..];
    let Some(at) = after_scheme.find('@') else {
        return target.to_string();
    };
    format!(
        "{}***@{}",
        &target[..scheme_end + 3],
        &after_scheme[at + 1..]
    )
}

#[cfg(test)]
mod tests {
    use super::redact_destination_target;

    #[test]
    fn redact_destination_target_strips_userinfo() {
        assert_eq!(
            redact_destination_target("https://user:secret@example.test/reg"),
            "https://***@example.test/reg"
        );
        assert_eq!(redact_destination_target("/tmp/dist"), "/tmp/dist");
    }
}
