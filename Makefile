RUSTC ?= rustc
RUSTFLAGS ?= -O

BINARY ?= beamwand

SRC ?=
SRC += src/cli.rs
SRC += src/beam.rs

all: $(BINARY)

clean:
	rm -f $(BINARY)

$(BINARY): $(SRC)
	$(RUSTC) $(RUSTFLAGS) src/cli.rs -o $@


.PHONY: all clean
