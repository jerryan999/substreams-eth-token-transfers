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

.PHONY: sink_files
sink_files: build
	./substreams-sink-files run --encoder=lines --file-working-dir="$(ROOT_DIR)/sink-files/working" --state-store="$(ROOT_DIR)/sink-files/workdir/state.yaml" $(ENDPOINT) "$(ROOT_DIR)/substreams.yaml" csv_out "$(ROOT_DIR)/out_csv" "16300000:16301000" 

.PHONY: package
package: build
	substreams pack -o substreams.spkg ./substreams.yaml
