cid := bafk2bzaced7lpqffeqjtwnctxiarkq5egskx6uvvofhfsn6xdwsft7b65xbnq
address := t01008
invoke := lotus chain invoke
.PHONY: install-actor
build:
	cargo build

install-actor:
	lotus chain install-actor ./target/debug/wbuild/frc20/frc20.compact.wasm 

create-actor:
	lotus chain create-actor $(cid) ""

mint:
	$(invoke) $(address) 2 ""

balance_of:
	$(invoke) $(address) 3 ""

allowance:
	$(invoke) $(address) 4 ""

transfer_from:
	$(invoke) $(address) 5 ""
	
transfer:
	$(invoke) $(address) 6 ""

approve:
	$(invoke) $(address) 7 ""
