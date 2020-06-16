# Simple automation for running tests and examples

RUSTFLAGS="-C link-args=-Tlink.x"
TARGET=thumbv7em-none-eabihf
EXAMPLES=target/$(TARGET)/release/examples
OBJDUMP=arm-none-eabi-objdump
OBJCOPY=arm-none-eabi-objcopy

TEENSY_CLI=teensy_loader_cli
TEENSY_CLI_ARGS=-v -w --mcu=TEENSY40

ifneq (, $(shell which $(TEENSY_CLI)))
	LOADER=$(shell which $(TEENSY_CLI)) $(TEENSY_CLI_ARGS)
else
	LOADER=echo
endif

# Build all the examples
.PHONY: examples
examples:
	@RUSTFLAGS=$(RUSTFLAGS) cargo build --examples --target $(TARGET) --release

# Build and flash a Teensy 4 example
.PHONY: t4_%
t4_%: examples
	@$(OBJCOPY) -O ihex -R .eeprom $(EXAMPLES)/$@ $(EXAMPLES)/$@.hex
	@$(OBJDUMP) -D -C $(EXAMPLES)/$@ > $(EXAMPLES)/$@.lst
	@$(OBJDUMP) -t -C $(EXAMPLES)/$@ > $(EXAMPLES)/$@.sym
	@$(LOADER) $(EXAMPLES)/$@.hex

# Run valid crate tests
.PHONY: test
test:
	@cargo test --lib
	@cargo test --doc

.PHONY: clean
clean:
	@cargo clean