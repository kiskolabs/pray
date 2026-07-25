.PHONY: build clean install bench bench-scaling ruby-test libyears libyears-rust libyears-ruby libyears-npm bump-homebrew

HOMEBREW_TAP ?= $(abspath ../../amkisko/homebrew-tap)
VERSION ?= $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
TAG ?= v$(VERSION)

build:
	cargo build --workspace

clean:
	cargo clean

install:
	cargo install --path crates/pray-cli --locked

ruby-test:
	cd rubygems/pray-cli && bundle install && bundle exec rspec

bench:
	cargo bench -p pray-bench

bench-scaling:
	cargo test -p pray-bench -- --ignored --nocapture

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
