use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use rust_indexed::index::SearchIndex;
use rust_indexed::parsers::{parse_html_page, parse_md_page, parse_summary_md};
use rust_indexed::{INDEX_CODE_DIR, INDEX_PAGE_DIR};

#[derive(Debug, Deserialize)]
struct IndexedSource {
    title: String,
    base_url: String,
    directory: String,
    is_html: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct Config {
    sources: Vec<IndexedSource>,
}

fn main() -> tantivy::Result<()> {
    let config: Config =
        toml::from_str(&std::fs::read_to_string("config.toml")?).expect("No config.toml");

    let mut index_page = SearchIndex::create(INDEX_PAGE_DIR)?;
    let mut index_code = SearchIndex::create(INDEX_CODE_DIR)?;

    let mut total_pages = 0;
    let mut total_code_blocks = 0;

    // For each mdbook
    for source in config.sources {
        if source.is_html == Some(true) {
            println!("Indexing {:?}", source.directory);

            if let Ok(files) = fs::read_dir(&source.directory) {
                for file in files {
                    let file_name = file?.file_name();
                    let path = PathBuf::from(&source.directory).join(&file_name);
                    println!("Indexing {:?}", &path);

                    let buf = std::fs::read_to_string(&path)?;
                    let (content, _code_blocks, chapter_title) = parse_html_page(&buf);

                    let url = format!("{}/{}", source.base_url, file_name.to_str().unwrap());
                    let title = format!("{} - {}", chapter_title.unwrap(), source.title);

                    index_page.add_document(url.clone(), title.clone(), content)?;

                    total_pages += 1;
                }
            }
        } else {
            let path = PathBuf::from(&source.directory).join("SUMMARY.md");
            println!("Indexing {:?}", path);
            let buf = std::fs::read_to_string(&path)?;

            // Parse chapters from SUMMARY.md
            let summary_md = parse_summary_md(&buf);

            // For each chapter
            for (chapter_title, rel_url) in summary_md {
                let url = format!("{}/{}.html", source.base_url, rel_url);
                let title = format!("{} - {}", chapter_title, source.title);

                // Index chapter title
                // index_mdbook.add_document(url.clone(), title.clone(), String::new())?;

                let mut path = PathBuf::from(&source.directory).join(format!("{}.md", rel_url));
                println!("Indexing {:?}", &path);

                if let Ok(buf) = std::fs::read_to_string(&path) {
                    path.pop();

                    // Index chapter md page
                    let (content, code_blocks) =
                        parse_md_page(buf.as_str(), path.to_str().unwrap());
                    index_page.add_document(url.clone(), title.clone(), content)?;

                    // Index code blocks found in the chapter
                    for code_block in code_blocks {
                        index_code
                            .add_document(url.clone(), title.clone(), code_block)
                            .unwrap();
                        total_code_blocks += 1;
                    }
                    total_pages += 1;
                } else {
                    eprintln!("Couldn't parse {:?}", path);
                }
            }
        }
        println!(
            "Indexed {} pages and {} code blocks",
            total_pages, total_code_blocks
        );
    }

    index_page.commit()?;
    index_code.commit()?;

    Ok(())
}
