.SUFFIXES:

.PHONY: help
help:
	@echo 'Targets:'
	@echo '    make install   # installs the binary'
	@echo '    make test      # runs all tests'
	@echo '    make fuzz      # starts the fuzzer'
	@echo '    make bench     # runs all benchmarks'
	@echo '    make help      # shows this help'

.PHONY: install
install:
	cargo install --path bin

.PHONY: test
test:
	cargo xtask test

FUZZ_J = 1
.PHONY: fuzz
fuzz:
	cd lib && cargo fuzz run -j ${FUZZ_J} round_trip

.PHONY: bench
bench:
	cargo xtask test --task=bench

.PHONY: clean
clean:
	git clean -fxd
