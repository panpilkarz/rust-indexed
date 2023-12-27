use serde::Deserialize;
use std::path::PathBuf;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::{doc, ReloadPolicy, SnippetGenerator};

use rust_indexed::indexers::SearchIndex;
use rust_indexed::parsers::{parse_md_page, parse_summary_md};

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
    let config: Config =
        toml::from_str(&std::fs::read_to_string("config.toml")?).expect("No config.toml");

    let mut summary_md_index = SearchIndex::new("index_summary_md")?;
    let mut page_index = SearchIndex::new("index_page")?;
    let mut code_block_index = SearchIndex::new("index_code_block")?;

    // For each mdbook
    for source in config.sources {
        let path = PathBuf::from(&source.directory).join("SUMMARY.md");
        println!("Indexing {:?}", path);
        let buf = std::fs::read_to_string(&path)?;

        // Parse chapters from SUMMARY.md
        let summary_md = parse_summary_md(&buf);
        let mut total_pages = 0;
        let mut total_code_blocks = 0;

        // For each chapter
        for (chapter_title, rel_url) in summary_md {
            let url = format!("{}/{}.html", source.base_url, rel_url);
            let title = format!("{} - {}", chapter_title, source.title);

            // Index chapter title
            summary_md_index.add_document(title.clone(), url.clone(), String::new())?;

            let mut path = PathBuf::from(&source.directory).join(format!("{}.md", rel_url));
            if let Ok(buf) = std::fs::read_to_string(&path) {
                path.pop();

                // Index chapter md page
                let (content, code_blocks) = parse_md_page(buf.as_str(), path.to_str().unwrap());
                page_index.add_document(title.clone(), url.clone(), content)?;

                // Index code blocks found in the chapter
                for code_block in code_blocks {
                    code_block_index
                        .add_document(title.clone(), url.clone(), code_block)
                        .unwrap();
                    total_code_blocks += 1;
                }
                total_pages += 1;
            } else {
                eprintln!("Couldn't parse {:?}", path);
            }
        }
        println!(
            "Indexed {} pages and {} code blocks",
            total_pages, total_code_blocks
        );
    }

    summary_md_index.commit()?;
    page_index.commit()?;
    code_block_index.commit()?;

    for r in summary_md_index.search("await") {
        println!("{:?}", r);
    }
    for r in page_index.search("await") {
        println!("{:?}", r);
    }
    code_block_index.search("await");

    Ok(())
}
