.SUFFIXES:

.PHONY: build
build:
	cargo build --release

.PHONY: encode
encode:
	cargo test --example encode --release

.PHONY: test
test:
	cargo test --release

.PHONY: doc
doc:
	cargo doc

.PHONY: bench
bench: encode
	rustc -V | grep -v nightly >/dev/null || cargo bench
	./bench.sh

.PHONY: update-doc
update-doc: doc
	./update-doc.sh

.PHONY: clean
clean:
	rm -rf Cargo.lock target
