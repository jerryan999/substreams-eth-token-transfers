ENDPOINT ?= mainnet.eth.streamingfast.io:443
ROOT_DIR := $(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))

.PHONY: build
build:
	cargo build --target wasm32-unknown-unknown --release

.PHONY: stream_csv
stream_csv: build
	substreams run -e $(ENDPOINT) substreams.yaml csv_out -s 12292922 -t +10

.PHONY: stream_jsonl
stream_jsonl: build
	substreams run -e $(ENDPOINT) substreams.yaml jsonl_out -s 12292922 -t +10


.PHONY: codegen
codegen:
	substreams protogen ./substreams.yaml --exclude-paths="sf/substreams,google"

# for eth
.PHONY: sink_files_eth
sink_files_eth: build
	substreams-sink-files run --encoder=lines --file-working-dir="$(ROOT_DIR)/sink-files/working" --state-store="$(ROOT_DIR)/sink-files/workdir/state.yaml" mainnet.eth.streamingfast.io:443 "$(ROOT_DIR)/substreams.yaml" jsonl_out "$(ROOT_DIR)/chain-transfer" "$(START_BLOCK):-1" 


# for bsc
.PHONY: sink_files_bsc
sink_files_bsc: build
	substreams-sink-files run --encoder=lines --file-working-dir="$(ROOT_DIR)/sink-files-bsc/working" --state-store="$(ROOT_DIR)/sink-files-bsc/workdir/state.yaml" bnb.streamingfast.io:443 "$(ROOT_DIR)/substreams.yaml" jsonl_out "$(ROOT_DIR)/chain-transfer-bsc" "$(START_BLOCK):-1" 

# for polygon
.PHONY: sink_files_polygon
sink_files_polygon: build
	substreams-sink-files run --encoder=lines --file-working-dir="$(ROOT_DIR)/sink-files-polygon/working" --state-store="$(ROOT_DIR)/sink-files-polygon/workdir/state.yaml" polygon.streamingfast.io:443 "$(ROOT_DIR)/substreams.yaml" jsonl_out "$(ROOT_DIR)/chain-transfer-polygon" "$(START_BLOCK):-1" 


.PHONY: package
package: build
	substreams pack -o substreams.spkg ./substreams.yaml
