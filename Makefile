# Makefile for a Rust project

# Define the Cargo commands
CARGO_CHECK = cargo check
CARGO_BUILD = cargo build
CARGO_TEST = cargo test

# Define targets
all: check build test

check:
	@echo "Running cargo check..."
	$(CARGO_CHECK)

build:
	@echo "Running cargo build..."
	$(CARGO_BUILD)

test:
	@echo "Running cargo test..."
	$(CARGO_TEST)

.PHONY: all check build test
