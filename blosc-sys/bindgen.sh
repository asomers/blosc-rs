#! /bin/sh
bindgen --no-rustfmt-bindings \
	--blocklist-type __uint64_t \
	--blocklist-type __size_t \
	--allowlist-type '.*BLOSC.*' \
	--allowlist-function '.*blosc.*' \
	--allowlist-var '.*BLOSC.*' /usr/local/include/blosc.h > src/bindgen.rs
rustfmt src/bindgen.rs
