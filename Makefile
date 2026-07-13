.PHONY: build clean install bench bench-scaling ruby-test

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
