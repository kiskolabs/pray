/// Peak resident set in bytes when the platform reports it.
pub fn peak_rss_bytes() -> Option<u64> {
    #[cfg(unix)]
    {
        let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
        let result = unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) };
        if result != 0 {
            return None;
        }
        let rss = unsafe { usage.assume_init() }.ru_maxrss;
        if rss <= 0 {
            return None;
        }
        let rss = rss as u64;
        #[cfg(target_os = "macos")]
        let bytes = rss;
        #[cfg(not(target_os = "macos"))]
        let bytes = rss.saturating_mul(1024);
        Some(bytes)
    }
    #[cfg(not(unix))]
    {
        None
    }
}

pub fn cpu_time_nanos() -> Option<u128> {
    #[cfg(unix)]
    {
        let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
        let result = unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) };
        if result != 0 {
            return None;
        }
        let usage = unsafe { usage.assume_init() };
        Some(timeval_nanos(usage.ru_utime) + timeval_nanos(usage.ru_stime))
    }
    #[cfg(not(unix))]
    {
        None
    }
}

#[cfg(unix)]
fn timeval_nanos(time: libc::timeval) -> u128 {
    (time.tv_sec as u128)
        .saturating_mul(1_000_000_000)
        .saturating_add((time.tv_usec as u128).saturating_mul(1_000))
}
