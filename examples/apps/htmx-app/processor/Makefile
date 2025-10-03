all: build

TARGET_DIR=$(HOME)/.cargo/target
PROJECT_VERSION=$(shell git rev-parse --short HEAD)

build:
	cargo build
	@cp $(TARGET_DIR)/debug/play play
