.PHONY: run
run: test
	cargo run

.PHONY: build
build: 
	cargo build

.PHONY: debug
debug: 
	rust-gdb target/debug/chip8

.PHONY: test
test:
	cargo test
