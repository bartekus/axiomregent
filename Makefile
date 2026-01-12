# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Bartek Kus
# axiomregent Makefile

SHELL := /bin/bash

.PHONY: rust-build rust-test rust-lint rust-fmt-check check build test lint fmt-check

check: fmt-check lint test build;

build: rust-build

test: rust-test

lint: rust-lint

fmt-check: rust-fmt-check

rust-fmt-check:
	@echo "Checking Rust formatting..."
	@cargo fmt --check

rust-lint:
	@echo "Linting Rust..."
	@cargo clippy -p axiomregent --all-targets -- -D warnings

rust-test:
	@echo "Testing Rust..."
	@cargo test -p axiomregent

rust-build:
	@echo "Building Rust..."
	@cargo build -p axiomregent --release && cp ./target/release/axiomregent ./bin/axiomregent
