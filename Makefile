RUSTC ?= rustc
RUSTFLAGS ?=

BINARY ?= ./beamwand
TEST_BINARY ?= ./beamwand_test

SRC ?=
SRC += src/cli.rs
SRC += src/beam.rs

all: $(BINARY)

test: $(TEST_BINARY)
	$(TEST_BINARY)

clean:
	rm -f $(BINARY) $(TEST_BINARY)

$(BINARY): $(SRC)
	$(RUSTC) $(RUSTFLAGS) src/cli.rs -o $@

$(TEST_BINARY): $(SRC)
	$(RUSTC) $(RUSTFLAGS) --test src/cli.rs -o $@


.PHONY: all test clean
