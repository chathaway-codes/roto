.PHONY: help test check test-issues test-rust test-no-std check-nostd fuzz bench issues issues-ready issues-ui clean docker-build docker-test docker-shell docker-yolo

PROJECT_NAME ?= roto
DOCKER_IMAGE ?= agent-workspace:latest
CONTAINER_NAME ?= agy-roto
DATA_VOLUME ?= agy-data-$(PROJECT_NAME)

help: ## Show this help message
	@echo "Available targets:"
	@grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}'

test: test-issues test-rust test-no-std ## Run test suite

check: test ## Alias for test

test-issues: ## Run unit tests for git issue tracker CLI and server
	python3 tests/test_issue_tooling.py

test-rust: ## Run Cargo workspace unit and integration tests (including optional tokio transport)
	cargo test --workspace
	cargo test -p roto-grpc --features tokio

test-no-std: ## Verify bare-metal #![no_std] and zero-allocation compilation (Cortex-M4F & RP2040 Cortex-M0+)
	cargo build --manifest-path tests/no_std_check/Cargo.toml --target thumbv7em-none-eabihf
	cargo build --manifest-path tests/no_std_check/Cargo.toml --target thumbv6m-none-eabi
	cargo check -p roto-runtime -p roto -p roto-grpc -p google-protobuf-roto-bootstrap -p google-protobuf-roto -p grpc-reflection-roto --target thumbv7em-none-eabihf
	cargo check -p roto-runtime -p roto -p roto-grpc -p google-protobuf-roto-bootstrap -p google-protobuf-roto -p grpc-reflection-roto --target thumbv6m-none-eabi

check-nostd: test-no-std ## Alias for test-no-std

fuzz: ## Run deterministic wire, data, descriptor, HPACK, HTTP/2, and gRPC fuzzers
	ROTO_FUZZ_ITERS=5000 ROTO_FUZZ_BATCHES=5 cargo test --workspace --test fuzz_wire --test fuzz_data --test fuzz_descriptors --test fuzz_hpack --test fuzz_h2 --test fuzz_grpc

bench: ## Run protobuf serialization/deserialization benchmarks (roto vs native C upb vs prost)
	cargo run --release --manifest-path proto-benchmarks/Cargo.toml

issues: ## List git issues in tabular format
	@python3 scripts/issue.py list

issues-ready: ## List open issues ready for pickup (no blockers)
	@python3 scripts/issue.py ready

issues-ui: ## Launch read-only issue browser Web UI on port 8080
	python3 scripts/issue_server.py --port 8080

clean: ## Clean generated test caches and temporary files
	rm -rf __pycache__ tests/__pycache__ scripts/__pycache__

docker-build: ## Build containerized development Docker image
	docker build -t $(DOCKER_IMAGE) .

docker-test: ## Run test suite inside container
	docker run --rm -v $(CURDIR):/workspace -w /workspace $(DOCKER_IMAGE) make test

docker-shell: ## Launch interactive shell inside container
	docker run --rm -it \
		-v $(CURDIR):/workspace \
		-v $(DATA_VOLUME):/home/developer/.gemini/antigravity-cli \
		-v $(HOME)/.gemini/antigravity-cli/antigravity-oauth-token:/home/developer/.gemini/antigravity-cli/antigravity-oauth-token:ro \
		-w /workspace \
		$(DOCKER_IMAGE) /bin/bash

docker-yolo: ## Launch containerized Antigravity daemon in isolated YOLO mode
	docker rm -f $(CONTAINER_NAME) 2>/dev/null || true
	docker run -d --name $(CONTAINER_NAME) \
		-v $(CURDIR):/workspace \
		-v $(DATA_VOLUME):/home/developer/.gemini/antigravity-cli \
		-v $(HOME)/.gemini/antigravity-cli/antigravity-oauth-token:/home/developer/.gemini/antigravity-cli/antigravity-oauth-token:ro \
		-w /workspace \
		$(DOCKER_IMAGE) \
		bash -c '\
			mkdir -p ~/.gemini/config/projects ~/.gemini/antigravity-cli && \
			printf "%s\n" "{\"userSettings\": {\"cliRemoteControlHostname\": \"$(PROJECT_NAME)-sandbox\", \"remoteControlHostname\": \"$(CONTAINER_NAME)\", \"autoExecutionPolicy\": \"always-proceed\", \"globalPermissionGrants\": {\"allow\": [\"command(*)\", \"read_url(*)\", \"mcp(*)\"]}}}" > ~/.gemini/config/config.json && \
			printf "%s\n" "{\"id\": \"$(PROJECT_NAME)\", \"name\": \"$(PROJECT_NAME)\", \"projectResources\": {\"resources\": [{\"gitFolder\": {\"folderUri\": \"file:///workspace\", \"defaultBranch\": \"main\"}}]}, \"settings\": {}, \"isWorkspaceOnly\": false}" > ~/.gemini/config/projects/$(PROJECT_NAME).json && \
			printf "%s\n" "{\"autoExecutionPolicy\": \"always-proceed\", \"trustedWorkspaces\": [\"/workspace\", \"/tmp\", \"/home/developer\"]}" > ~/.gemini/antigravity-cli/settings.json && \
			exec agy --dangerously-skip-permissions remote-control serve'
