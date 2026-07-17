.PHONY: build clean install bench bench-scaling ruby-test libyears libyears-rust libyears-ruby libyears-npm

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
