run:
	cargo run
x86:
	cargo build --target x86_64-unknown-linux-musl
x86-release:
	cargo build --target x86_64-unknown-linux-musl --release
arm:
	cargo build --target armv7-unknown-linux-gnueabihf
arm-release:
	cargo build --target armv7-unknown-linux-gnueabihf --release
arm-musl:
	RUSTFLAGS="-C target-feature=-crt-static" cargo build --target armv7-unknown-linux-musleabihf
arm-musl-2:
	 CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER=arm-linux-gnueabihf-gcc CC_armv7_unknown_linux_musleabihf=arm-linux-gnueabihf-gcc cargo build --target armv7-unknown-linux-musleabihf
cross-arm:
	cross build --target armv7-unknown-linux-gnueabihf
cross-arm-musl:
	cross build --target armv7-unknown-linux-musleabihf
arm64:
	cargo build --target aarch64-unknown-linux-gnu
train:
	rm -rf ./actionengine
	snips-nlu train actiondata.json actionengine
	rm -rf ./chatengine
	snips-nlu train chatdata.json chatengine