.PHONY: all build test contracts backend clean

all: build

build: contracts backend

contracts:
	cd contracts && cargo build --target wasm32-unknown-unknown --release

test-contracts:
	cd contracts && cargo test

backend:
	cd server && npm run build

backend-dev:
	cd server && npm run dev

install:
	cd server && npm install

clean:
	cd contracts && cargo clean
	rm -rf server/dist
