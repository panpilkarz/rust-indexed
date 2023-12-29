index: 
	rm -rf index_summary_md/*
	rm -rf index_page/*
	rm -rf index_code_block/*
	cargo run --example indexer

test:
	cargo test --lib
