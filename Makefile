all:
	cargo build

create:
	- cat json/create.json | cargo run create

setup:
	- cat json/setup-teardown.json | cargo run setup /tmp/

teardown:
	- cat json/setup-teardown.json | cargo run teardown /tmp/

run_all: all create setup teardown


.phony: all create setup teardown run_all