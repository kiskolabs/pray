# Fuzz targets for pray-core parsers

Requires nightly Rust and `cargo-fuzz`:

```bash
cargo install cargo-fuzz
cargo +nightly fuzz run parse_manifest
cargo +nightly fuzz run parse_package_spec
cargo +nightly fuzz run validate_package_path
```

These targets are not part of the workspace `cargo test` / CI job. CI covers the same surfaces with `proptest` in `crates/pray-core/tests/parser_fuzz.rs`.
