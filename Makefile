.SUFFIXES:

.PHONY: help
help:
	@echo 'Targets:'
	@echo '    make install   # installs the binary'
	@echo '    make bench     # runs all benchmarks'
	@echo '    make test      # runs all tests'
	@echo '    make help      # shows this help'

.PHONY: install
install:
	cd bin && cargo install

.PHONY: bench
bench:
	rustc -V | grep -v nightly >/dev/null || { cd lib && cargo bench; }
	rustc -V | grep -v nightly >/dev/null || { cd cmp && cargo bench; }
	cd bin && ./bench.sh

.PHONY: test
test:
	cargo test --all
	rustc -V | grep -v nightly >/dev/null || { \
	  cd lib/macro && cargo test --no-default-features; }
	cd bin && ./test.sh

.PHONY: clean
clean:
	git clean -fxd
