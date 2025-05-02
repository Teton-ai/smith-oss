dev.docs:
	cd docs && mdbook serve --open

lint:
	cargo fmt
	cargo clippy --release --all-targets --all-features -- -D clippy::all

fix:
	cargo fix --allow-dirty
