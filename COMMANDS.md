```
[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
```
```
[target.armv7-unknown-linux-musleabihf]
linker = "arm-linux-gnueabihf-gcc"
```
```
snips-nlu train path/to/data.json path/to/output_folder
```
```
scp -P 5914 target/armv7-unknown-linux-gnueabihf/release/terminal_alpha_beta alarm@192.168.0.104:terminal
```
```
scp -P 5914 target/armv7-unknown-linux-musleabihf/release/terminal_alpha_beta alarm@192.168.0.104:terminal
```
```
scp -P 5914 data/responses.json  alarm@192.168.0.104:responses.json
```
```
scp -P 5914 -r data/rootengine  alarm@192.168.0.104:rootengine
```

```
dependencies on arch/manjaro for
------
arm-musl-release:
	 CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER=arm-linux-gnueabihf-gcc CC_armv7_unknown_linux_musleabihf=arm-linux-gnueabihf-gcc cargo build --target armv7-unknown-linux-musleabihf --release
```
```
Needed
------
> arm-linux-gnueabihf-gcc
>		arm-linux-gnueabihf-binutils
>		arm-linux-gnueabihf-gcc-stage1
>		arm-linux-gnueabihf-linux-api-headers
>		arm-linux-gnueabihf-glibc-headers
>		arm-linux-gnueabihf-gcc-stage2
>		arm-linux-gnueabihf-glibc
>		arm-linux-gnueabihf-gcc
> clang
> openssl (? was already installed, shouldn't risk uninstall testing, probably important to OS itself)
```
```
Needed for training intent
------
> Python 3.7+
> `pip install snips-nlu`
> `python -m snips_nlu download en` or simply `snips-nlu download en`
```
```
Not Needed (Probably, further testing required)
------
> arm-linux-gnueabihf-musl
> crfsuite (unsure - test by removing)
```