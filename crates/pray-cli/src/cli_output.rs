use std::env;
use std::sync::atomic::{AtomicU32, Ordering};

static VERBOSE_LEVEL: AtomicU32 = AtomicU32::new(0);

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
    env::var_os("PRAY_QUIET").is_some_and(|value| value != "0")
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
