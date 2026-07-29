#![no_main]

use libfuzzer_sys::fuzz_target;
use pray_core::package_spec::parse_package_spec;

fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);
    let _ = parse_package_spec(&input);
});
