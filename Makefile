.PHONY: init
init:
	./scripts/init.sh

.PHONY: check
check:
	SKIP_WASM_BUILD= cargo check

.PHONY: test
test:
	SKIP_WASM_BUILD= cargo test --all

.PHONY: run
run:
	cargo run --release -- --dev --tmp -lruntime=debug

.PHONY: build
build:
	cargo build --release  --features with-bitcountry-runtime

.PHONY: check-debug
check-debug:
	RUSTFLAGS="-Z macro-backtrace" SKIP_WASM_BUILD= cargo +nightly check

.PHONY: build-docker
build-docker:
	./scripts/docker_run.sh

.PHONY: run-dev
run-dev:
	./target/release/bitcountry-node --dev --tmp -lruntime=debug
