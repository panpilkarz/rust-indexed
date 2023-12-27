index: 
	rm -rf index_summary_md/*
	rm -rf index_page/*
	cargo run

test:
	cargo test --lib
