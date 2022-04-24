cid := 0
address := t01003
.PHONY: install-actor
build:
	cargo build

install-actor:
	lotus chain install-actor ./target/debug/wbuild/frc20/frc20.compact.wasm 

create-actor:
	lotus chain create-actor $(cid) aGVsbG8K

balance_of:
	lotus chain invoke $(address) 2 aGVsbG8K