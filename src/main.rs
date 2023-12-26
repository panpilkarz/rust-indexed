use serde::{Deserialize, Serialize};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy};
use toml;

#[derive(Debug, Deserialize)]
struct IndexedSource {
    title: String,
    base_url: String,
    directory: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    sources: Vec<IndexedSource>,
}

fn main() -> tantivy::Result<()> {
    println!("Hello, world!");

    let config: Config =
        toml::from_str(&std::fs::read_to_string("config.toml")?).expect("Invalid toml");

    dbg!(config);
    std::process::exit(0);

    let mut schema_builder = Schema::builder();

    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT | STORED);

    let documents = [
        include_str!("../mdbooks/comprehensive-rust/src/SUMMARY.md"),
        include_str!("../mdbooks/yet-another-rust-resource/src/SUMMARY.md"),
        include_str!("../mdbooks/rust-book/first-edition/src/SUMMARY.md"),
        include_str!("../mdbooks/rust-book/second-edition/src/SUMMARY.md"),
        include_str!("../mdbooks/rust-book/src/SUMMARY.md"),
        include_str!("../mdbooks/rustonomicon/src/SUMMARY.md"),
        include_str!("../mdbooks/rust-by-example/src/SUMMARY.md"),
    ];

    let schema = schema_builder.build();
    let title = schema.get_field("title").unwrap();
    let body = schema.get_field("body").unwrap();

    let index = Index::create_in_dir("index", schema.clone())?;

    let mut index_writer: IndexWriter = index.writer(50_000_000)?;

    for doc in documents {
        for line in doc.split("\n") {
            println!("{line}");
            index_writer.add_document(doc!(
                title => line.to_string(),
                body => "body".to_string(),
            ))?;
        }
    }

    index_writer.commit()?;

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::Manual)
        .try_into()?;

    let searcher = reader.searcher();

    let query_parser = QueryParser::for_index(&index, vec![title, body]);

    let query = query_parser.parse_query("generics")?;

    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        let json = schema.to_json(&retrieved_doc);
        println!("{}", json);
    }

    Ok(())
}
