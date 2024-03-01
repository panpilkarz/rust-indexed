all:
	cargo run

index: reset
	cargo run --example indexer

run-prod:
	cargo run --release

test:
	cargo test --lib

bench:
	ab -n 100 -c 10 "127.0.0.1:3000/search/?q=await"

curl:
	curl "127.0.0.1:3000/search/?q=async&page=0" | jq .

ci:
	cargo fmt --all
	cargo check
	cargo clippy --profile test --all-features -- -D warnings

get-mdbooks:
	mkdir -p mdbooks
	cd mdbooks; git clone git@github.com:google/comprehensive-rust.git || true
	cd mdbooks; git clone git@github.com:rust-lang/book.git || true
	cd mdbooks; git clone git@github.com:rust-lang/nomicon.git || true
	cd mdbooks; git clone git@github.com:tokio-rs/website.git || true
	cd mdbooks; git clone git@github.com:rust-lang/rust-by-example.git || true
	cd mdbooks; git clone git@github.com:rust-lang-nursery/rust-cookbook.git || true
	cd mdbooks; git clone https://github.com/panpilkarz/rust-vs-python || true
	cd mdbooks; git clone https://git.sr.ht/~ntietz/yet-another-rust-resource || true

update-mdbooks:
	git -C mdbooks/comprehensive-rust pull
	git -C mdbooks/book pull
	git -C mdbooks/nomicon pull
	git -C mdbooks/rust-by-example pull
	git -C mdbooks/rust-cookbook pull
	git -C mdbooks/rust-vs-python pull
	git -C mdbooks/yet-another-rust-resource pull
	git -C mdbooks/website pull
	rm -rf mdbooks/website/content/tokio/tutorial/index.md

reset:
	rm -rf indexes/*
	mkdir -p indexes/page indexes/code
