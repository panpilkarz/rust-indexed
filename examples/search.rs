use rust_indexed::ranking::{Ranking, SearchFlags};

fn main() -> tantivy::Result<()> {
    let needle = std::env::args().nth(1).expect("usage: $1 phrase");

    let ranking = Ranking::new();

    for r in ranking.search(&needle, SearchFlags::DEFAULT) {
        dbg!(r);
    }

    Ok(())
}
