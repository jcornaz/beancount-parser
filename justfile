set dotenv-load

@_choose:
	just --list --unsorted

# Perform all verifications (compile, test, lint, etc.)
verify: test lint doc check-msrv
	cargo deny check licenses

# Watch the source files and run `just verify` when source changes
watch:
	cargo watch --delay 0.1 --clear --why -- just verify

# Run the tests
test:
	cargo hack test --feature-powerset

# Run the static code analysis
lint:
	cargo fmt -- --check
	cargo hack clippy --all-targets

# Build the documentation
doc *args:
	cargo doc --all-features --no-deps {{args}}

# Open the documentation page
doc-open: (doc "--open")

# Make sure the MSRV is satisfiable
check-msrv:
	cargo msrv verify

# Clean up compilation output
clean:
	rm -rf target
	rm -f Cargo.lock
	rm -rf node_modules

# Install cargo dev-tools used by the `verify` recipe (requires rustup to be already installed)
install-dev-tools:
	rustup install stable
	rustup override set stable
	cargo install cargo-hack cargo-watch cargo-msrv cargo-deny cargo-release

release *args: verify
	cargo release {{args}}

