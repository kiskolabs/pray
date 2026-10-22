## Dependency

pray-cli Ruby gem and npm package (same repository, versions through 1.22.0). Rust CLI was not affected.

## Symptom

Local `pray publish` to a distribution root rewrote every selected package catalog JSON and dropped `derived_metadata` from historical version rows. New rows also lacked derived metadata. Downstream `make catalog-topics` then had nothing to normalize for new versions.

## Evidence

- Ruby `RegistryPackageVersion` and TypeScript `RegistryPackageVersion` omitted `derived_metadata`.
- `version_from_hash` / `versionFromHash` and `version_to_hash` / `versionToHash` whitelisted known fields only.
- Unchanged publish still called `write_registry_package_metadata`, so a lossy round trip rewrote every selected catalog.
- Rust `published_registry_package_version` already derived and serialized the field; `RegistryPackageVersion::merge_annotations_from` preserved it.
- Tip-only storage of derived metadata is an open size note in `docs/issues/20261007192126_package-json-version-growth.md`, not the active schema contract.

## Suggested fix

Carry `derived_metadata` through Ruby and TypeScript load and write paths. Derive it for new publish rows from the package root. Keep historical rows intact when appending a version. Covered by publish specs that republish and append.

## Next

- Release the Ruby and npm packages with the fix.
- Downstream publishers that already restored catalogs from Git can republish with the fixed gem or npm CLI, or keep using the Rust binary.

## Source

Downstream report against pray-cli 1.21.0 used as `pray` from the repository Makefile.
Upstream tree: `rubygems/pray-cli`, `npmjs/pray-cli`, `crates/pray-cli/src/publish_version.rs`.
