#![no_main]

use libfuzzer_sys::fuzz_target;
use pray_core::paths::validate_package_relative_path;
use std::path::Path;

fuzz_target!(|data: &[u8]| {
    let input = String::from_utf8_lossy(data);
    let _ = validate_package_relative_path(Path::new(input.as_ref()));
});
