.PHONY: build clean install

build:
	cargo build --workspace

clean:
	cargo clean

install:
	cargo install --path crates/pray-cli --locked
