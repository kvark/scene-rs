.PHONY: all lib aster update run

all: lib aster

lib:
	cargo build

aster:
	(cd examples/asteroids && cargo build)

update:
	cargo update
	(cd examples/asteroids && cargo update)

run: aster
	examples/asteroids/target/asteroids
