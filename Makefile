.PHONY: dev prod check release

dev:
	cargo run -p pixel-spaceships-app --features dev

prod:
	cargo run -p pixel-spaceships-app --release

check:
	cargo fmt --all --check
	cargo clippy --all-targets -- -D warnings
	cargo test

release:
	@if [ -z "$(v)" ]; then echo "Error: Version parameter is required. Use 'make release v=x.y.z'"; exit 1; fi
	@grep -q '^version = "$(v)"$$' Cargo.toml || { echo "Error: workspace version in Cargo.toml does not match $(v)"; exit 1; }
	$(MAKE) check
	git tag v$(v)
	git push origin v$(v)
