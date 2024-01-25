.PHONY: init
init:
	./scripts/init.sh

.PHONY: check
check: githooks
	SKIP_WASM_BUILD= cargo check --features with-metaverse-runtime

.PHONY: check-pioneer
check-pioneer: githooks
	SKIP_WASM_BUILD= cargo check --features with-pioneer-runtime

.PHONY: check-continuum
check-continuum: githooks
	SKIP_WASM_BUILD= cargo check --features with-continuum-runtime

.PHONY: check-all
check-all: githooks
	SKIP_WASM_BUILD= cargo check --features with-pioneer-runtime,with-metaverse-runtime

.PHONY: check-debug
check-debug:
	RUSTFLAGS="-Z macro-backtrace" SKIP_WASM_BUILD= cargo +nightly check --features with-metaverse-runtime

.PHONY: check-formatting
check-formatting:
	cargo fmt --all -- --check

.PHONY: test
test:
	SKIP_WASM_BUILD= cargo test --all --features with-pioneer-runtime,with-metaverse-runtime

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

.PHONY: build-continuum
build-continuum:
	cargo build --release  --features with-continuum-runtime

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
	./target/release/metaverse-node --dev --tmp --alice --node-key 0000000000000000000000000000000000000000000000000000000000000001 -lruntime=debug

.PHONY: run-bob-dev
run-bob-dev:
	./target/release/metaverse-node --dev --tmp --bob --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp -lruntime=debug

.PHONY: spawn-zombienet-basic
spawn-zombienet-basic:
	./scripts/zombienet/zombienet.sh spawn_basic

.PHONY: spawn-zombienet-metaverse
spawn-zombienet-metaverse:
	./scripts/zombienet/zombienet.sh spawn_metaverse

.PHONY: spawn-zombienet-pioneeer
spawn-zombienet-meatverse:
	./scripts/zombienet/zombienet.sh spawn_pioneer

.PHONY: install-chopsticks
install-chopsticks:
	npm i -g @acala-network/chopsticks@latest

.PHONY: run-chopsticks-pioneer
run-chopsticks-pioneer:
	npx @acala-network/chopsticks --config=scripts/chopsticks/chopsticks_pioneer.yml

.PHONY: run-chopsticks-pioneer-xcm
run-chopsticks-pioneer-xcm:
	npx @acala-network/chopsticks xcm -r kusama -p statemine -p scripts/chopsticks/chopsticks_pioneer.yml

GITHOOKS_SRC = $(wildcard githooks/*)
GITHOOKS_DEST = $(patsubst githooks/%, .git/hooks/%, $(GITHOOKS_SRC))

.git/hooks:
	mkdir .git/hooks

.git/hooks/%: githooks/%
	cp $^ $@

.PHONY: githooks
githooks: .git/hooks $(GITHOOKS_DEST)