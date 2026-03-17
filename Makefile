.PHONY: dev check lint fmt

dev:
	cargo run

check:
	cargo check

lint:
	cargo clippy -- -D clippy::pedantic -D clippy::nursery -D clippy::cargo -A clippy::multiple_crate_versions

fmt:
	cargo fmt
