.PHONY: init
init:
	./scripts/init.sh

.PHONY: check
check: githooks
	SKIP_WASM_BUILD= cargo check --features with-metaverse-runtime

.PHONY: check-tewai
check-tewai: githooks
	SKIP_WASM_BUILD= cargo check --features with-tewai-runtime

.PHONY: check-pioneer
check-pioneer: githooks
	SKIP_WASM_BUILD= cargo check --features with-pioneer-runtime

.PHONY: check-all
check-all: githooks
	SKIP_WASM_BUILD= cargo check --features with-pioneer-runtime with-metaverse-runtime with-tewai-runtime

.PHONY: check-debug
check-debug:
	RUSTFLAGS="-Z macro-backtrace" SKIP_WASM_BUILD= cargo +nightly check --features with-metaverse-runtime

.PHONY: test
test:
	SKIP_WASM_BUILD= cargo test --all

.PHONY: run
run:
	cargo run --release -- --dev --tmp -lruntime=debug

.PHONY: build
build:
	cargo build --release  --features with-metaverse-runtime

.PHONY: build-tewai
build-tewai:
	cargo build --release  --features with-tewai-runtime

.PHONY: build-docker
build-docker:
	./scripts/docker_run.sh

.PHONY: build-docker-tewai
build-docker-tewai:
	./scripts/docker_build_tewai.sh

.PHONY: run-dev
run-dev:
	./target/release/metaverse-node purge-chain --dev
	./target/release/metaverse-node --dev --tmp -lruntime=debug

GITHOOKS_SRC = $(wildcard githooks/*)
GITHOOKS_DEST = $(patsubst githooks/%, .git/hooks/%, $(GITHOOKS_SRC))

.git/hooks:
	mkdir .git/hooks

.git/hooks/%: githooks/%
	cp $^ $@

.PHONY: githooks
githooks: .git/hooks $(GITHOOKS_DEST)