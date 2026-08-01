.PHONY: all build clean install format test lint

all: build

build:
	cargo build --release

format:
	cargo fmt --all
	npx oxfmt@latest --write "web/**/*.mjs" "web/**/*.html" "web/**/*.css" "*.md" "build.mjs"

test:
	cargo test

lint:
	cargo clippy --all-targets -- -D warnings
	npx oxfmt@latest --check "web/**/*.mjs" "web/**/*.html" "web/**/*.css" "*.md" "build.mjs"

clean:
	cargo clean

install:
	install -d $(DESTDIR)/usr/local/bin
	install -m 755 target/release/il2dump $(DESTDIR)/usr/local/bin/il2dump

wasm:
	PATH="/opt/homebrew/opt/rustup/bin:$$PATH" cargo build --target wasm32-unknown-unknown --release
	wasm-bindgen --target web --out-dir web/pkg --out-name il2dump-lib target/wasm32-unknown-unknown/release/il2dump_lib.wasm
	wasm-opt -Oz -o web/pkg/il2dump-lib_bg.wasm web/pkg/il2dump-lib_bg.wasm
	npm run build

web-server:
	python3 -m http.server 8080 --directory dist
