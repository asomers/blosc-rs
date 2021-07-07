#! /bin/sh
bindgen --no-rustfmt-bindings \
	--blacklist-type __uint64_t \
	--blacklist-type __size_t \
	--whitelist-type '.*BLOSC.*' \
	--whitelist-function '.*blosc.*' \
	--whitelist-var '.*BLOSC.*' /usr/local/include/blosc.h > src/bindgen.rs
rustfmt src/bindgen.rs
