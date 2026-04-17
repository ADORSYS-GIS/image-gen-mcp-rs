export USER_ID := `id -u`
export GROUP_ID := `id -g`

prepare: # Build the backend binary in release mode
	cargo build --release

all-checks:
	@echo "Running Rust formatting, lint, and checks"
	cargo fmt
	cargo deny check
	cargo fix --allow-dirty
	cargo clippy --all-targets --all-features --fix --allow-dirty -- -D warnings
	cargo check --all-targets --all-features
