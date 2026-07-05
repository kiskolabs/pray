.PHONY: build clean install bench bench-scaling

build:
	cargo build --workspace

clean:
	cargo clean

install:
	cargo install --path crates/pray-cli --locked

bench:
	cargo bench -p pray-bench

bench-scaling:
	cargo test -p pray-bench -- --ignored --nocapture
