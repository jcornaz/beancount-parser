toolchain := ""
_toolchain_arg := if toolchain == "" { "" } else { "+" + toolchain }
deny-warnings := "true"
export RUSTFLAGS := if deny-warnings == "true" { "-D warnings" } else { "" }
export RUSTDOCFLAGS := RUSTFLAGS

@_choose:
	just --choose --unsorted

# Perform all verifications (compile, test, lint, etc.)
verify: doc lint test

# Watch changes, and run `just verify` when source changes
watch:
	cargo watch -s 'just verify'

# Install cargo dev-tools used by other recipes (requires rustup to be already installed)
install-dev-tools:
	rustup install stable
	rustup override set stable
	cargo install cargo-hack cargo-watch

# Install a git hook to run tests before every commits
install-git-hooks:
	echo "#!/usr/bin/env sh" > .git/hooks/pre-commit
	echo "just verify" >> .git/hooks/pre-commit
	chmod +x .git/hooks/pre-commit

# Run all tests
test:
	cargo {{_toolchain_arg}} hack test --feature-powerset

# Static code analysis
lint:
	cargo {{_toolchain_arg}} fmt -- --check
	cargo {{_toolchain_arg}} clippy --all-features --all-targets

# Build the documentation
doc *args:
	cargo {{_toolchain_arg}} doc --all-features --no-deps {{args}}

# Open the documentation page
doc-open: (doc "--open")

# Clean up compilation output
clean:
	rm -rf target
	rm -f Cargo.lock
	rm -rf node_modules

# run the release process in dry run mode (requires npm and a `GITHUB_TOKEN`)
release-dry-run: (release "--dry-run")

# Run the release process (requires `npm`, a `GITHUB_TOKEN` and a `CARGO_REGISTRY_TOKEN`)
release *args:
	npm install --no-save conventional-changelog-conventionalcommits @semantic-release/exec
	npx semantic-release {{args}}
