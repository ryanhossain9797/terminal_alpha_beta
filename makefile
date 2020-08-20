run:
	cargo run
x86:
	cargo build --target x86_64-unknown-linux-musl
x86-release:
	cargo build --target x86_64-unknown-linux-musl --release
# Works
arm:
	cargo build --target armv7-unknown-linux-gnueabihf
# Works
arm-release:
	cargo build --target armv7-unknown-linux-gnueabihf --release
# Doesn't work
arm-release-static:
	RUSTFLAGS="-C target-feature=-crt-static" cargo build --target armv7-unknown-linux-gnueabihf --release
# Doesn't work
arm-musl:
	RUSTFLAGS="-C target-feature=-crt-static" cargo build --target armv7-unknown-linux-musleabihf
# Works
arm-musl-2:
	 CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER=arm-linux-gnueabihf-gcc CC_armv7_unknown_linux_musleabihf=arm-linux-gnueabihf-gcc cargo build --target armv7-unknown-linux-musleabihf --release
cross-arm:
	cross build --target armv7-unknown-linux-gnueabihf
cross-arm-musl:
	cross build --target armv7-unknown-linux-musleabihf
arm64:
	cargo build --target aarch64-unknown-linux-gnu
train:
	rm -rf ./data/rootengine
	rm -f trainingdata/actiondata.json
	rm -f trainingdata/chatdata.json
	snips-nlu generate-dataset en trainingdata/chatdata.yaml trainingdata/actiondata.yaml > trainingdata/rootdata.json
	snips-nlu train trainingdata/rootdata.json data/rootengine
go-lib:
	env GOOS=linux GOARCH=arm CGO_ENABLED=1 CC=arm-linux-gnueabihf-gcc go build -buildmode=c-archive -o libpeople.a main.go