.PHONY: init
init:
	./scripts/init.sh

.PHONY: check
check: githooks
	SKIP_WASM_BUILD= cargo check --features with-metaverse-runtime

.PHONY: check-pioneer
check-pioneer: githooks
	SKIP_WASM_BUILD= cargo check --features with-pioneer-runtime

.PHONY: check-all
check-all: githooks
	SKIP_WASM_BUILD= cargo check --features with-pioneer-runtime with-metaverse-runtime

.PHONY: check-debug
check-debug:
	RUSTFLAGS="-Z macro-backtrace" SKIP_WASM_BUILD= cargo +nightly check --features with-metaverse-runtime

.PHONY: check-formatting
check-formatting:
	cargo fmt --all -- --check

.PHONY: test
test:
	SKIP_WASM_BUILD= cargo test --all

.PHONY: test-metaverse
test-metaverse:
	SKIP_WASM_BUILD= cargo test --all --features with-metaverse-runtime

.PHONY: test-pioneer
test-pioneer:
	SKIP_WASM_BUILD= cargo test --all --features with-pioneer-runtime

.PHONY: run
run:
	cargo run --release -- --dev --tmp -lruntime=debug

.PHONY: build
build:
	cargo build --release  --features with-metaverse-runtime

.PHONY: build-pioneer
build-pioneer:
	cargo build --release  --features with-pioneer-runtime

.PHONY: build-benchmarking
build-benchmarking:
	cargo build --release --features runtime-benchmarks

.PHONY: build-docker
build-docker:
	./scripts/docker_run.sh

.PHONY: build-docker-pioneer
build-docker-pioneer:
	./scripts/docker_build_pioneer.sh

.PHONY: run-dev
run-dev:
	./target/release/metaverse-node purge-chain --dev
	./target/release/metaverse-node --dev --tmp --alice -lruntime=debug

GITHOOKS_SRC = $(wildcard githooks/*)
GITHOOKS_DEST = $(patsubst githooks/%, .git/hooks/%, $(GITHOOKS_SRC))

.git/hooks:
	mkdir .git/hooks

.git/hooks/%: githooks/%
	cp $^ $@

.PHONY: githooks
githooks: .git/hooks $(GITHOOKS_DEST)