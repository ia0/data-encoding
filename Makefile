.SUFFIXES:

.PHONY: help
help:
	@echo 'Targets:'
	@echo '    make install   # installs the binary'
	@echo '    make bench     # runs all benchmarks'
	@echo '    make test      # runs all tests'
	@echo '    make doc       # generates the library documentation'
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
	cd bin && ./test.sh

.PHONY: doc
doc:
	cd lib && cargo doc

.PHONY: clean
clean:
	git clean -fxd
