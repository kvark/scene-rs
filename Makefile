.PHONY: all lib examples

all: lib examples

lib:
	cargo build

examples:
	(cd src/examples/asteroids && cargo build)
