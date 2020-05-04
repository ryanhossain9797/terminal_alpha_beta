run:
	cargo run
build:
	cargo build --target x86_64-unknown-linux-musl
build-release:
	cargo build --target x86_64-unknown-linux-musl --release
build-arm:
	cargo build --target armv7-unknown-linux-gnueabihf
build-arm-musl:
	RUSTFLAGS="-C target-feature=+crt-static" cargo build --target armv7-unknown-linux-musleabihf
cross-arm:
	cross build --target armv7-unknown-linux-gnueabihf
cross-arm-musl:
	cross build --target armv7-unknown-linux-musleabihf