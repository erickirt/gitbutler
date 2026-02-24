.PHONY: check fmt clippy all

all: fmt fmt-check clippy

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check --all

clippy:
	cargo clippy --workspace --all-targets -- -D warnings
