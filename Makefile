# Makefile for Teensy project

# Variables
TARGET = leadscrew-teensy
MCU = TEENSY41
BUILD_DIR = target/thumbv7em-none-eabihf/release

# Default target
all: build $(TARGET).hex upload

# Build the Rust project
build:
	cargo build --release

# Build the hex file
$(TARGET).hex: $(BUILD_DIR)/$(TARGET)
	rust-objcopy -O ihex $< $@

# Upload to Teensy
upload: $(TARGET).hex
	teensy_loader_cli --mcu=$(MCU) -w $<

# Clean up
clean:
	rm -f $(TARGET).hex

.PHONY: all upload clean
