BINARY_NAME ?= socktail
AUTH_KEY ?=
CONTROL_URL ?=

PLATFORMS = \
	x86_64-unknown-linux-gnu \
	aarch64-unknown-linux-gnu \
	x86_64-apple-darwin \
	aarch64-apple-darwin

.PHONY: build
build:
	cargo build --release

.PHONY: build-with-key
build-with-key:
	@if [ -z "$(AUTH_KEY)" ]; then \
		echo "Error: AUTH_KEY is required. Usage: make build-with-key AUTH_KEY=... [CONTROL_URL=...]"; \
		exit 1; \
	fi
	AUTH_KEY="$(AUTH_KEY)" CONTROL_URL="$(CONTROL_URL)" cargo build --release

.PHONY: build-all-with-key
build-all-with-key:
	@if [ -z "$(AUTH_KEY)" ]; then \
		echo "Error: AUTH_KEY is required. Usage: make build-all-with-key AUTH_KEY=... [CONTROL_URL=...]"; \
		exit 1; \
	fi
	@mkdir -p dist
	@for target in $(PLATFORMS); do \
		echo "Building $$target"; \
		rustup target add $$target >/dev/null 2>&1 || true; \
		AUTH_KEY="$(AUTH_KEY)" CONTROL_URL="$(CONTROL_URL)" cargo build --release --target $$target || exit 1; \
		cp target/$$target/release/socktail-rust-ofuscated dist/$(BINARY_NAME)-$$target || true; \
	done

.PHONY: fmt
fmt:
	cargo fmt --all

.PHONY: lint
lint:
	cargo clippy --all-targets -- -D warnings

.PHONY: test
test:
	cargo test
