# Resolve filesystem reduction

## Decisions

The compat bench resolve reads Prayfile, one prayspec, and one package file. Hash and export already share the package bytes from that one read. An earlier profile spent most of its time in the kernel on open, lstat, and readdir.

The attempt kept a directory descriptor for each path inside one resolve, listed a package directory once, and cached file bytes so a second read of the same path would not reopen it. Package path components still used O_NOFOLLOW. On Darwin, openat of a directory symlink with O_DIRECTORY and O_NOFOLLOW returns ENOTDIR, so the attempt also used fstatat with AT_SYMLINK_NOFOLLOW before treating ENOTDIR as absence. A real directory did not take that branch.

The descriptor cache is not kept. The no-follow symlink_metadata walk in source_path.rs stays. Symlink rejection is unchanged.

## Effects

Measurements used cargo run --release --offline -p pray-bench --example compat_bench on Darwin arm64. The harness warms up 20 times, takes 500 samples, and prints p50_us as the mean of the two central samples. That value is median_us below. The fixture is one path package, file exports/a.md, composed into A.md. The release pray size is from cargo build --release --offline -p pray-cli, Mach-O arm64, stat of the cargo-target release binary.

Baseline, no-follow lstat walk, three runs. resolve median_us 74.98, 70.77, 71.75. The middle value is 71.75. parse 5.12, 5.21, 5.46. render 1.54, 1.54, 1.54. lock 8.25, 8.29, 8.17. Release pray: 8892000 bytes.

Openat directory cache, three runs. resolve median_us 90.92, 80.35, 86.29. The middle value is 86.29. parse 5.33, 5.31, 7.46. render 1.58, 1.54, 1.58. lock 8.33, 8.46, 8.42. Release pray: 8908480 bytes, 16480 bytes above the baseline.

The middle resolve sample moved from 71.75 us to 86.29 us. That is slower, and the gap is larger than the about 10 us run-to-run spread already recorded for this fixture. The fixture has one directory listing and one file. The byte cache did not remove a read, because hash and export already shared one buffer. Replacing lstat with openat plus a path map still does one syscall per component, then pays for the map and the held descriptors. On a warm directory cache, lstat is cheaper than that bookkeeping.

After the cache was removed, three runs. resolve median_us 69.62, 65.06, 69.33. The middle value is 69.33. parse 5.29, 5.17, 6.04. render 1.62, 1.54, 1.54. lock 8.75, 8.21, 8.21. Release pray: 8891856 bytes. That sits with the baseline, inside the same spread.

cargo fmt -p pray-core completed. cargo clippy -p pray-core --offline --tests -- -D warnings completed with exit 0. cargo test -p pray-core --offline completed with exit 0. The library reported 133 passed. source_symlink reported 4 passed. One test was ignored.

## Next

A listing cache or a byte cache can pay for itself when one resolve lists a large directory more than once, or reads the same package file more than once. This fixture does neither.

Do not replace the no-follow lstat walk with openat on this path unless a larger fixture shows a gain that clears the 10 us spread.

## Source

docs/issues/20261026114500_source-symlink-rejection.md
crates/pray-core/src/source_path.rs
crates/pray-core/src/resolve_exports.rs
crates/pray-bench/examples/compat_bench.rs
