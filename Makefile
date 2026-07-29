.PHONY: build clean install bench bench-scaling ruby-test loc-check libyears libyears-rust libyears-ruby libyears-npm bump-homebrew \
	release-dry-run release-crates release-npm release-rubygems release-distribution release-all \
	coverage coverage-rust mutants fuzz-build

HOMEBREW_TAP ?= $(abspath ../../amkisko/homebrew-tap)
VERSION ?= $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
TAG ?= v$(VERSION)
ROOT ?= ./prayers
SERVER ?=
SIGNING_KEY ?=

build:
	cargo build --workspace

clean:
	cargo clean

install:
	cargo install --path crates/pray-cli --locked --force

ruby-test:
	cd rubygems/pray-cli && bundle install && make test

# Soft warn at 150 LOC; hard fail above 300 unless ratcheted in scripts/loc-limits.allowlist.
loc-check:
	./scripts/check-loc-limits.sh

bench:
	cargo bench -p pray-bench

bench-scaling:
	cargo test -p pray-bench -- --ignored --nocapture

coverage: coverage-rust

coverage-rust:
	cargo llvm-cov --workspace --summary-only --fail-under-lines 20

mutants:
	cargo mutants -p pray-core --timeout 60 \
		-f 'manifest.rs' -f 'package_spec.rs' -f 'dependency_graph.rs' \
		-f 'resolve.rs' -f 'hashing.rs' -f 'package_integrity.rs' \
		-f 'package_archive.rs' -f 'paths.rs'

fuzz-build:
	cargo +nightly fuzz build --fuzz-dir fuzz

libyears: libyears-rust libyears-ruby libyears-npm

libyears-rust:
	cargo-libyear --sort libyear --top 10

libyears-ruby:
	cd rubygems/pray-cli && bundle exec rake libyears

libyears-npm:
	cd npmjs/pray-cli && npm run libyears

# Requires the GitHub tag to exist. Updates sibling amkisko/homebrew-tap.
bump-homebrew:
	@test -n "$(VERSION)" || (echo "could not read version from Cargo.toml" >&2; exit 2)
	$(HOMEBREW_TAP)/scripts/bump-formula.sh \
		--formula pray \
		--tag "$(TAG)" \
		--repository kiskolabs/pray \
		--commit

# Manual release helpers. Language registries default to dry-run / build-only.
# Use scripts/release/*.sh --publish (or make release-* with care) to push.
release-dry-run:
	./scripts/release/all.sh --dry-run

release-crates:
	./scripts/release/crates.sh --publish

release-npm:
	./scripts/release/npm.sh --publish

release-rubygems:
	./scripts/release/rubygems.sh --publish

release-distribution:
	@args="--root $(ROOT)"; \
	if [ -n "$(SERVER)" ]; then args="$$args --server $(SERVER)"; fi; \
	if [ -n "$(SIGNING_KEY)" ]; then args="$$args --signing-key $(SIGNING_KEY)"; fi; \
	./scripts/release/distribution.sh $$args

release-all:
	@args="--publish --root $(ROOT)"; \
	if [ -n "$(SERVER)" ]; then args="$$args --server $(SERVER)"; fi; \
	if [ -n "$(SIGNING_KEY)" ]; then args="$$args --signing-key $(SIGNING_KEY)"; fi; \
	./scripts/release/all.sh $$args
