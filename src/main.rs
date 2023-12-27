use serde::Deserialize;
use std::path::PathBuf;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::{doc, ReloadPolicy, SnippetGenerator};

use rust_indexed::indexers::{PageIndex, SummaryMdIndex};
use rust_indexed::parsers::parse_summary_md;

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

    let mut summary_md_index = SummaryMdIndex::new("index_summary_md")?;
    let mut page_index = PageIndex::new("index_page")?;

    for source in config.sources {
        let path = PathBuf::from(&source.directory).join("SUMMARY.md");
        println!("Indexing {:?}", path);
        let buf = std::fs::read_to_string(path.to_str().unwrap())?;
        let summary_md = parse_summary_md(&buf);
        let mut total = 0;
        for (desc, rel_url) in summary_md {
            let url = format!("{}/{}.html", source.base_url, rel_url);
            let title = format!("{} - {}", desc, source.title);
            summary_md_index.add_document(title.clone(), url.clone())?;

            let path = PathBuf::from(&source.directory).join(format!("{}.md", rel_url));
            if let Ok(buf) = std::fs::read_to_string(path.to_str().unwrap()) {
                page_index.add_document(title, url, buf)?;
                total += 1;
            }
            else {
                eprintln!("Couldn't parse {:?}", path);
            }
        }
        println!("Indexed {} pages", total);
    }

    summary_md_index.commit()?;
    page_index.commit()?;

    // std::process::exit(0);

    // let index = &summary_md_index.index;
    // let title = &summary_md_index.title;
    // let url = &summary_md_index.url;

    let index = &page_index.index;
    let body = &page_index.body;
    let url = &page_index.url;

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::Manual)
        .try_into()?;

    let searcher = reader.searcher();

    // let query_parser = QueryParser::for_index(index, vec![*title]);
    let query_parser = QueryParser::for_index(index, vec![*body]);

    let query = query_parser.parse_query("async")?;

    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    let snippet_generator = SnippetGenerator::create(&searcher, &*query, *body)?;

    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        // let title = retrieved_doc.get_first(*title).unwrap().as_text().unwrap();
        // let body = retrieved_doc.get_first(*body).unwrap().as_text().unwrap();
        let snippet = snippet_generator.snippet_from_doc(&retrieved_doc).to_html().replace("\n", " ");
        let url = retrieved_doc.get_first(*url).unwrap().as_text().unwrap();
        println!("{} snippet={} url={}", _score, snippet, url);
    }

    Ok(())
}
