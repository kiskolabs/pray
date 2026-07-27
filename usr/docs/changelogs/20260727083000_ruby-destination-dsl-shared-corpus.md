# Ruby destination DSL and shared fixture corpus

## Participants

- Andrei Makarov

## Decisions

- Port compose, tree, and file: destination parse plus scoped render and provision into the Ruby gem so runtime behavior matches Rust and TypeScript.
- Keep a small shared fixture corpus under testdata/shared/manifest, with thin loaders in each implementation suite; rely on existing CI jobs rather than a fourth corpus-only workflow.

## Effects

- Ruby parses compose, tree, folder, skills, output, and file blocks; binds destination entries; selects exports by role; renders scoped compose without legacy Shared instructions fan-out; materializes exact file: bindings and tree-scoped skills.
- Manifest JSON encoding includes destination fields so Ruby manifest_hash tracks Rust for simple-project.
- Shared compose-tree-file fixture is asserted by Rust, TypeScript, and Ruby tests.

## Next

- Grow the shared corpus when further destination edge cases need cross-runtime lock-in.

## Source

- rubygems/pray-cli/lib/pray/destination.rb
- rubygems/pray-cli/lib/pray/manifest.rb
- rubygems/pray-cli/lib/pray/render.rb
- rubygems/pray-cli/lib/pray/resolve.rb
- testdata/shared/
- crates/pray-core/tests/shared_corpus.rs
- npmjs/pray-cli/src/shared-corpus.test.ts
- rubygems/pray-cli/spec/pray/shared_corpus_spec.rb
- rubygems/pray-cli/spec/pray/destination_render_spec.rb
