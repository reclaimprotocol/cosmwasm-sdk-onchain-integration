all: 
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown
	cp target/wasm32-unknown-unknown/release/reclaim_cosmwasm.wasm reclaim_cosmwasm.wasm

secr: 
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --no-default-features --features secret
	cp target/wasm32-unknown-unknown/release/reclaim_cosmwasm.wasm reclaim_cosmwasm.wasm
	cat ./reclaim_cosmwasm.wasm | gzip -9 > ./reclaim_cosmwasm.wasm.gz
	
vanilla:
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --no-default-features --features vanilla
	cp target/wasm32-unknown-unknown/release/reclaim_cosmwasm.wasm reclaim_cosmwasm.wasm
	cat ./reclaim_cosmwasm.wasm | gzip -9 > ./reclaim_cosmwasm.wasm.gz

test:
	cargo +nightly test -- --nocapture
clean:
	cargo clean
	rm reclaim_cosmwasm.wasm reclaim_cosmwasm.wasm.gz
