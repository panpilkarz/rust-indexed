use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, Schema, STORED, TEXT};
use tantivy::{doc, Index, IndexWriter, ReloadPolicy, Searcher, SnippetGenerator, TantivyError};

pub struct SearchIndex {
    dir: String,
    index: Index,
    // optionals
    searcher: Option<Searcher>,
    index_writer: Option<IndexWriter>,
    query_parser: Option<QueryParser>,
}

#[derive(Debug)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub snippet: String,
}

impl SearchIndex {
    pub fn create(dir: &str) -> Result<Self, TantivyError> {
        let schema = SearchIndex::create_schema();
        let index = Index::create_in_dir(dir, schema)?;
        let index_writer = index.writer(50_000_000)?;

        println!("Created `{dir}` directory");

        Ok(Self {
            dir: dir.to_string(),
            index,
            searcher: None,
            index_writer: Some(index_writer),
            query_parser: None,
        })
    }

    pub fn open(dir: &str) -> Result<Self, TantivyError> {
        let index = Index::open_in_dir(dir)?;

        println!("Opened `{dir}` index");

        Ok(Self {
            dir: dir.to_string(),
            index,
            searcher: None,
            index_writer: None,
            query_parser: None,
        })
    }

    fn create_schema() -> Schema {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("url", TEXT | STORED);
        schema_builder.add_text_field("body", TEXT | STORED);

        schema_builder.build()
    }

    fn url(&self) -> Field {
        self.index.schema().get_field("url").unwrap()
    }

    fn title(&self) -> Field {
        self.index.schema().get_field("title").unwrap()
    }

    fn body(&self) -> Field {
        self.index.schema().get_field("body").unwrap()
    }

    pub fn add_document(
        &mut self,
        url: String,
        title: String,
        body: String,
    ) -> Result<u64, TantivyError> {
        self.index_writer.as_ref().unwrap().add_document(doc!(
            self.url() => url,
            self.title() => title,
            self.body() => body,
        ))
    }

    pub fn commit(&mut self) -> Result<u64, TantivyError> {
        self.index_writer.as_mut().unwrap().commit()?;

        println!("Commited `{}` index", self.dir);

        Ok(0)
    }

    pub fn search(&mut self, query: &str) -> Result<Vec<SearchResult>, TantivyError> {
        if self.searcher.is_none() {
            // TODO: lazy?
            let reader = self
                .index
                .reader_builder()
                .reload_policy(ReloadPolicy::Manual)
                .try_into()?;

            self.searcher = Some(reader.searcher());

            self.query_parser = Some(QueryParser::for_index(
                &self.index,
                vec![self.url(), self.title(), self.body()],
            ));
        }

        let mut results = Vec::new();

        if let Some(query_parser) = &self.query_parser {
            if let Ok(query) = query_parser.parse_query(query) {
                if let Some(searcher) = &self.searcher {
                    if let Ok(docs) = searcher.search(&query, &TopDocs::with_limit(10)) {
                        if let Ok(snippet_generator) =
                            SnippetGenerator::create(searcher, &query, self.body())
                        {
                            println!("docs={}", docs.len());
                            for (_score, doc_address) in docs {
                                if let Ok(retrieved_doc) = searcher.doc(doc_address) {
                                    let url = retrieved_doc
                                        .get_first(self.url())
                                        .unwrap()
                                        .as_text()
                                        .unwrap()
                                        .to_string();

                                    let title = retrieved_doc
                                        .get_first(self.title())
                                        .unwrap()
                                        .as_text()
                                        .unwrap()
                                        .to_string();

                                    let snippet = snippet_generator
                                        .snippet_from_doc(&retrieved_doc)
                                        .to_html()
                                        .replace('\n', " ")
                                        .trim()
                                        .to_string();

                                    results.push(SearchResult {
                                        title,
                                        url,
                                        snippet,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(results)
    }
}
