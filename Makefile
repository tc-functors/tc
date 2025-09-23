all: build

TARGET_DIR=$(HOME)/.cargo/target
BIN_DIR=bin
PROJECT_VERSION=$(shell git rev-parse --short HEAD)

build:
	@mkdir -p $(BIN_DIR)
	RUSTUP_TOOLCHAIN=1.90 cargo build
	@cp $(TARGET_DIR)/debug/tc tc

x86_64-linux:
	rustup target add x86_64-unknown-linux-musl
	PKG_CONFIG_ALLOW_CROSS=1 OPENSSL_STATIC=true OPENSSL_DIR=~/opt/musl RUSTFLAGS='-C link-arg=-s' cargo build --release --target x86_64-unknown-linux-musl
	@mkdir -p $(BIN_DIR)
	cargo build --release
	@cp $(TARGET_DIR)/x86_64-unknown-linux-musl/release/tc $(BIN_DIR)/tc-x86_64-linux
	upx --best --lzma $(BIN_DIR)/tc-x86_64-linux

export PATH := $(HOME)/opt/apple/osxcross/target/bin:$(PATH)

x86_64-apple:
	export CC=o64-clang LIBZ_SYS_STATIC=1; cargo build --release --target x86_64-apple-darwin
	@cp $(TARGET_DIR)/x86_64-apple-darwin/release/tc $(BIN_DIR)/tc-x86_64-apple

aarch64-apple:
	rm -f bin/tc
	cargo build --release
	@cp target/release/tc $(BIN_DIR)/tc

docs:
	cd doc && make run

unit-test:
	cargo test --quiet -j 2 -- --test-threads=2

integration-test:
	cargo test --test integration_test --quiet

fmt:
	 cargo +nightly fmt

ssl-musl:
	sudo ln -s /usr/include/x86_64-linux-gnu/asm /usr/include/x86_64-linux-musl/asm
	mkdir ~/opt/musl
	wget https://github.com/openssl/openssl/archive/OpenSSL_1_1_1f.tar.gz
	tar zxvf OpenSSL_1_1_1f.tar.gz
	cd openssl-OpenSSL_1_1_1f/ && CC="musl-gcc -fPIE -pie" ./Configure --prefix=${HOME}/opt/musl --openssldir=${HOME}/musl/ssl linux-x86_64 no-shared no-async no-engine -DOPENSSL_NO_SECURE_MEMORY && make depend && make -j$(nproc) && make install

clean:
	rm -rf bin tc
