RUSTC ?= rustc
RUSTFLAGS ?=

BINARY ?= beamwand

SRC ?=
SRC += src/cli.rs
SRC += src/beam.rs

all: $(BINARY)

clean:
	rm -f $(BINARY)

$(BINARY): $(SRC)
	$(RUSTC) src/cli.rs -o $@


.PHONY: all clean
