use rust_indexed::indexers::SearchIndex;

fn main() -> tantivy::Result<()> {
    let needle = std::env::args().nth(1).expect("usage: $1 phrase");

    let mut page_index = SearchIndex::open("index_page")?;
    let mut summary_md_index = SearchIndex::open("index_summary_md")?;
    let mut code_block_index = SearchIndex::open("index_code_block")?;

    for r in summary_md_index.search(&needle).unwrap() {
        dbg!(r);
    }
    for r in page_index.search(&needle).unwrap() {
        dbg!(r);
    }
    for r in code_block_index.search(&needle).unwrap() {
        dbg!(r);
    }

    Ok(())
}
