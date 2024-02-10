all:
	cargo run

index: reset
	cargo run --example indexer

test:
	cargo test --lib

bench:
	ab -n 100 -c 10 "http://127.0.0.1:3000/search/?q=await"

curl:
	curl "http://127.0.0.1:3000/search/?q=async&page=0" | jq .

ci:
	cargo fmt --all
	cargo check
	cargo clippy --profile test --all-features -- -D warnings

reset:
	rm -rf indexes/*
	mkdir -p indexes/page indexes/code
