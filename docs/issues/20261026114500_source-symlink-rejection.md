# Source symlink rejection

## Decisions

RFC 0011 forbids symlinks in a v1 package and forbids reading undeclared files during install and render. pray-core followed symlinks when it read a declared package file and when it read a compose local. The outside file was hashed and, for a fragment, rendered. Ports already reject both inputs.

Declared package files and compose locals are read with a component walk from the package root or the project root. Each existing component is inspected with symlink metadata. A symlink fails as an integrity error. On Unix the leaf is opened with O_NOFOLLOW. A missing package file and a package directory keep the previous errors. An optional missing local still resolves as empty. Registry fetch and git materialize are unchanged. Archive unpack already rejects symlink members.

Declared package directories are walked from the project root through the declared relative path. symlink_metadata inspects each normal component, including the package directory itself. A symlink fails as an integrity error. A ParentDir component such as a path source ../pkgs is pushed and is not treated as a symlink. A missing component returns the joined path so a later missing-package error stays the same. The prayspec file in that directory is then read with the same no-follow file walk.

## Effects

Before the walk, crates/pray-core/tests/source_symlink.rs failed both tests. Resolve succeeded. The package export body was the file outside the package. The compose local body was the file outside the project.

After the walk, both tests pass. The errors contain the words symbolic link.

cargo fmt -p pray-core completed. cargo clippy -p pray-core --offline --tests -- -D warnings completed with exit 0. cargo test -p pray-core --offline completed with exit 0. The library reported 132 passed. source_symlink reported 2 passed. The run reported no failures. One test was ignored.

Release measurements used cargo run --release --offline -p pray-bench --example compat_bench on Darwin arm64. The harness warms up 20 times, takes 500 samples, and prints the median of the two central samples. The fixture is one path package, file exports/a.md with the two bytes A and a newline, composed into A.md.

Before the walk, one sample: parse median_us=5.12, resolve median_us=65.38, render median_us=1.62, lock median_us=8.42. A second sample on the same library: parse 5.42, resolve 76.10, render 1.67, lock 8.62.

A stage split beside that second sample, one sample each: read_manifest 11.42, parse_manifest 5.42, manifest_hash 1.83, read_spec 10.58, parse_spec 2.75, tree_hash_for_root 18.06. Resolve is the hot path. The time is reading Prayfile, the prayspec, and the one package file. Render stays near 1.6 us, so building markers is not the cost on this fixture. One resolve reads each of those files once and hashes the package bytes from that buffer.

After the walk, three samples: parse 5.38, 5.46, 5.54; resolve 76.94, 82.54, 85.19; render 1.58, 1.67, 1.62; lock 8.75, 8.50, 8.67. Resolve sits at the high end of the earlier 65 to 76 us band. The added directory metadata check and the no-follow open account for that. No separate speed edit was made. Changing marker buffers would not move a 1.6 us render against a resolve near 80 us, and the run-to-run spread on resolve is already about 10 us.

Release binary from cargo build --release --offline -p pray-cli, Mach-O arm64: 8858720 bytes before, 8875472 bytes after, a difference of 16752 bytes. pray-transport and pray-cli relinked because they depend on pray-core.

A later pass rejects a package-directory symlink. crates/pray-core/tests/source_symlink.rs path_package_directory_symlink_outside_project_is_rejected refuses packages/base when that directory is a symlink to a real package outside the project. The error contains the words symbolic link. path_source_parent_directory_resolves_when_it_is_not_a_symlink still resolves a real directory reached by source path ../pkgs. source_symlink reported 4 passed.

cargo test -p pray-core --offline completed with exit 0 after that pass. The library reported 133 passed. One test was ignored. An earlier run of the same suite failed tests/write_transaction.rs once with File exists on a temp directory name. A rerun passed, and the full suite above passed. That test does not read package paths.

Ruby and TypeScript reject declared package-file and compose-local symlink components. Their package-directory selection remains open.

## Next

RFC 0011 says symlinks are forbidden. It does not yet require the no-follow component walk that RFC 0033 requires before a destination read. One sentence there would make this check and the port rejections the same rule.

Rust rejects the package-directory symlink. Add the same regression and project-root component walk to Ruby and TypeScript.

## Source

rfcs/0011-prayspec-and-package.md sections 25, 26, and 27
rfcs/0033-provisioned-destination-safety.md
pray_compat docs/issues/20261026090741_compat-cross-implementation-audit.md finding R1
crates/pray-core/src/source_path.rs
crates/pray-core/src/resolve_exports.rs
crates/pray-core/src/resolve_local.rs
crates/pray-core/src/package_spec_hash.rs
crates/pray-core/tests/source_symlink.rs
crates/pray-bench/examples/compat_bench.rs
