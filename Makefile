.PHONY: init
init:
	./scripts/init.sh

.PHONY: check scaredy-cat scaredy-cat
.PHONY: test
test:
	SKIP_WASM_BUILD= cargo test --all

.PHONY: run
run:
	cargo run --release -- --dev --tmp -lruntime=debug --instant-sealing

.PHONY: build
build:
	cargo build --release

.PHONY: check-debug
check-debug:
	RUSTFLAGS="-Z macro-backtrace" SKIP_WASM_BUILD= cargo +nightly check

.PHONY: build-docker
build-docker:
	./scripts/docker_run.sh
