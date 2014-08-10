.PHONY: all lib examples update run

all: lib examples

lib:
	cargo build

examples:
	(cd src/examples/asteroids && cargo build)
	(cd src/examples/simple && cargo build)

update:
	cargo update
	(cd src/examples/asteroids && cargo update)
	(cd src/examples/simple && cargo update)

run: examples
	src/examples/asteroids/target/asteroids
