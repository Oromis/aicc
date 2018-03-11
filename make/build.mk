.PHONY: build flash run

self_dir = $(dir $(lastword $(MAKEFILE_LIST)))

include $(self_dir)/variables.mk

# Build for the target board by default (perform cross-compiling)
for_target ?= true

# Don't build for the host by default
for_host ?= false

# Sets the default build directory to the cargo target directory if the project type is rust
ifeq "$(project_type)" "rust"
build_dir ?= target/aarch64-unknown-linux-gnu/release
endif

# Builds a sub-project depending on it's type
build:
ifeq "$(project_type)" "rust"
# Rust project => Invoke cargo
ifeq "$(for_target)" "true"
	@cargo build --release --target aarch64-unknown-linux-gnu
endif
ifeq "$(for_host)" "true"
	@cargo build --release
endif
else
	$(error Project type $(project_type) not supported yet :|)
endif

ifdef exe
# Flashes the project to the target board, building it beforehand if necessary.
flash: build
	@scp $(build_dir)/$(exe) $(host):$(base_dir)

# Runs the project executable on the target board (using an SSH tunnel).
# Will block until the executable terminates.
# Will NOT re-build NOR re-flash the executable. To do this, call `make flash run`
run:
	@ssh $(host) "cd $(base_dir); sudo ./$(exe)"
endif

# Build, flash and run the project
all: build flash run
