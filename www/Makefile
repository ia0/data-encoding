.SUFFIXES:

.PHONY: help
help:
	@echo 'Targets:'
	@echo '    make html      # builds website'
	@echo '    make help      # shows this help'

.PHONY: html
html:
	rustc -V | grep nightly >/dev/null
	cargo build --target wasm32-unknown-unknown --release
	rsync -a --delete static/ html
	wasm-bindgen --no-typescript --no-modules \
	  target/wasm32-unknown-unknown/release/data_encoding_www.wasm \
	  --out-dir html

.PHONY: clean
clean:
	git clean -fxd
