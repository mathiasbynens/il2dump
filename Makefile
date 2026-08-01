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

WASM_OPT_VERSION = 131
LOCAL_BIN = $(shell pwd)/.bin
WASM_OPT_DIR = $(LOCAL_BIN)/binaryen-version_$(WASM_OPT_VERSION)
WASM_OPT = $(WASM_OPT_DIR)/bin/wasm-opt

# Download and extract wasm-opt locally if it does not exist.
$(WASM_OPT):
	@mkdir -p $(LOCAL_BIN)
	@echo "Downloading wasm-opt version $(WASM_OPT_VERSION)..."
	@if [ "$$(uname)" = "Darwin" ]; then \
		curl -L https://github.com/WebAssembly/binaryen/releases/download/version_$(WASM_OPT_VERSION)/binaryen-version_$(WASM_OPT_VERSION)-x86_64-macos.tar.gz | tar -xz -C $(LOCAL_BIN); \
	else \
		curl -L https://github.com/WebAssembly/binaryen/releases/download/version_$(WASM_OPT_VERSION)/binaryen-version_$(WASM_OPT_VERSION)-x86_64-linux.tar.gz | tar -xz -C $(LOCAL_BIN); \
	fi

wasm: $(WASM_OPT)
	PATH="/opt/homebrew/opt/rustup/bin:$$PATH" cargo build --target wasm32-unknown-unknown --release
	wasm-bindgen --target web --out-dir web/pkg --out-name il2dump-lib target/wasm32-unknown-unknown/release/il2dump_lib.wasm
	$(WASM_OPT) -O3 --strip-producers --converge --optimize-stack-ir -o web/pkg/il2dump-lib_bg.wasm web/pkg/il2dump-lib_bg.wasm
	npm run build

web-server:
	python3 -m http.server 8080 --directory dist
