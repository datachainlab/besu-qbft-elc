include Enclave.mk

.PHONY: all
all: $(Signed_RustEnclave_Name)

.PHONY: fmt
fmt:
	@cargo fmt --all && cd ./enclave && cargo fmt --all

.PHONY: docker
docker:
	@cd rust-sgx-sdk/dockerfile && docker build --no-cache -t datachainlab/sgx-rust:2004-1.1.5 -f Dockerfile.2004.nightly .

.PHONY: yrly
yrly:
	cd ./relayer && go build -o ../bin/yrly -tags customcert .

######## E2E test ########

LCP_BIN ?= ./bin/lcp
LCP_REPO ?= ./lcp
HARDHAT ?= npx hardhat
DOCKER_COMPOSE ?= docker compose
BESU ?= docker run -v "${PWD}":/work -w /work -it hyperledger/besu:24.3.0

.PHONY: gen-network
gen-network:
	$(BESU) operator generate-blockchain-config --config-file=qbftConfigFile.json --to=networkFiles --private-key-file-name=key
	./gen.sh

.PHONY: network
network:
	$(DOCKER_COMPOSE) --verbose up --wait -d || docker compose logs

.PHONY: network-down
network-down:
	$(DOCKER_COMPOSE) down --volumes --remove-orphans

.PHONY: lcp
lcp:
	$(MAKE) -C $(LCP_REPO) ./bin/lcp -B && mkdir -p ./bin && mv $(LCP_REPO)/bin/lcp ./bin/

.PHONY: deploy-contracts
deploy-contracts:
	$(HARDHAT) run ./scripts/deploy.js --network chain0
	$(HARDHAT) run ./scripts/deploy.js --network chain1

.PHONY: extract-abi
extract-abi:
	./scripts/extract-abi.sh

.PHONY: e2e-test
e2e-test: $(Signed_RustEnclave_Name) yrly lcp
	LCP_BIN=$(LCP_BIN) ./scripts/run_e2e_test.sh

.PHONY: clean
clean:
	@rm -f $(RustEnclave_Name) $(Signed_RustEnclave_Name) enclave/*_t.* lib/*.a
	@cd enclave && cargo clean
