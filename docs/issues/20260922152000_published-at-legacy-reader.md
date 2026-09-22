# Published_at reader rejects pre-1.14 catalogs

## Participants

vesan

## Decisions

Treat RFC 3339 strings and JSON null as reader migration forms for published_at. Keep the canonical write path as a JSON integer or omit the field. Keep the registry schema strict: strings and null remain invalid for new catalogs. Apply the same reader rules in the Rust, Ruby, and TypeScript libraries so the three CLIs accept the same catalog bytes.

RFC 0061 already allowed numeric strings and RFC 3339 strings. The Rust reader only implemented the numeric half. This change implements the RFC 3339 half and documents JSON null as absent because the 1.13 schema allowed string or null and serde emitted null for Option::None.

## Effects

Before the fix, deserializing RegistryPackageMetadata failed for published_at null and for RFC 3339 strings. A numeric string such as "1789389330" already succeeded. After the fix, null becomes absent, RFC 3339 becomes whole UTC seconds with fractional seconds truncated toward the preceding whole second, and the next metadata write emits the canonical integer or omits the field.

The schema still rejects those legacy forms. Producers still write integers.

The person-facing surface is the terminal. Commands that load package metadata, including yank, install, plan, and sync, print a registry metadata parse error and exit 2 on those catalogs before the fix, and continue the command after. No new screen, pointer, or accessibility surface was added. A human visual review of the success and error lines remains open.

## Next

Federation package-version payloads use the same deserializer; the transport test covers RFC 3339 and null there.

## Source

rfcs/0061-publish-version-preservation.md
rfcs/0060-distribution.md
rfcs/0100-conformance.md
crates/pray-core/src/registry_timestamp.rs
rubygems/pray-cli/lib/pray/registry.rb
npmjs/pray-cli/src/registry/index.ts
schema/registry.schema.json
https://github.com/kiskolabs/pray/issues/26

## Claims

C-001. Claim: RFC 0061 changed published_at from a string to a JSON integer in 1.14.0. Outcome supported. Evidence: CHANGELOG 1.14.0, commit 3875bf9, schema unixTimestampSeconds, and RegistryPackageVersion changing from Option of String to Option of u64.

C-002. Claim: the Rust reader accepts a numeric string and rejects null and RFC 3339. Outcome supported before this fix. Evidence: deserialize_optional_publish_timestamp matched Number and parse of u64, and mapped every other JSON value to a hard error. The pre-fix registry_reader test expected null to fail.

C-003. Claim: 1.13 schema allowed string or null. Outcome supported. Evidence: v1.13.0 schema/registry.schema.json listed type string or null with minLength 1.

C-004. Claim: 1.13 Rust serialized None as null. Outcome supported for the type, partial for published catalogs. Evidence: v1.13.0 RegistryPackageVersion used serde default on Option of String without skip_serializing_if, so None became JSON null on write. The publish path set Some of current_timestamp, and current_timestamp wrote Unix seconds as a decimal string. Null appears when a row stayed None and metadata was rewritten, or when a catalog followed the 1.13 schema and emitted null.

C-005. Claim: Ruby wrote Time.now.utc.iso8601 and TypeScript wrote Date toISOString up to 1.13. Outcome supported. Evidence: v1.13.0 rubygems/pray-cli/lib/pray/publish.rb and npmjs/pray-cli/src/publish/index.ts.

C-006. Claim: Ruby and TypeScript 1.14 plus already accept RFC 3339. Outcome supported. Evidence: Time.iso8601 in registry.rb, regex plus Date.parse in registry/index.ts, and their 1.14 tests. Both still rejected JSON null before this fix.

C-007. Claim: RFC 0061 readers MAY accept numeric strings or RFC 3339 strings, and no clause covered null. Outcome supported before this RFC edit. Evidence: rfcs/0061-publish-version-preservation.md migration paragraph. This change adds the null MAY.

C-008. Claim: RFC 0100 level 4 Publisher and RFC 0060 section 29 make a static tree a shared artifact. Outcome partially supported. RFC 0100 names level 4 Publisher as pack and publish to a static registry, and lists Rust, Ruby, and TypeScript as implementations that must fail the same fixtures. RFC 0060 section 29 says the registry may be a static file tree and that static hosting must be enough. The phrase shared artifact is the reporter's synthesis of that interoperability, not a quoted clause.

C-009. Claim: yank, install, plan, and sync all fail because they deserialize RegistryPackageVersion. Outcome supported. Evidence: yank, publish, sync, and registry fetch all parse RegistryPackageMetadata. Parse errors use kind registry metadata and exit 2.

C-010. Claim: fallout is the representation change in 3875bf9, issue 26, not the preservation skip. Outcome supported. Evidence: the same commit introduced the integer deserializer and the no-op publish check. Repeat publish tests still assert unchanged timestamps.

C-011. Claim: a 1.19.0 yank of demo 1.0.0 prints yanked demo 1.0.0 on the numeric-string catalog. Outcome partially supported. v1.19.0 yank prints yanked package version in root. The parse error text and exit 2 match PrayError Parse display.
