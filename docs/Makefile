all: build

build:
	mdbook build --dest-dir dist

run:
	mdbook serve --open --dest-dir dist

deps:
	cargo install mdbook --version 0.4.36
	cargo install mdbook-admonish
	cargo install mdbook-mermaid
	cargo install mdbook-toc
	mdbook-admonish install

clean:
	rm -rf dist
