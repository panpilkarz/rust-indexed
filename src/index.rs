use serde::Serialize;
use tantivy::collector::{Count, TopDocs};
use tantivy::query::{FuzzyTermQuery, QueryParser};
use tantivy::schema::{Field, Schema, Term, STORED, TEXT};
use tantivy::{doc, Index, IndexWriter, ReloadPolicy, Searcher, SnippetGenerator, TantivyError};

pub struct SearchIndex {
    dir: String,
    index: Index,
    // optionals
    searcher: Option<Searcher>,
    index_writer: Option<IndexWriter>,
    query_parser: Option<QueryParser>,
    // flags
    skip_snippet: bool,
    return_body: bool,
}

#[derive(Debug, Default, Serialize)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub snippet: Option<String>,
    pub body: Option<String>,
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
            skip_snippet: false,
            return_body: false,
            searcher: None,
            index_writer: Some(index_writer),
            query_parser: None,
        })
    }

    pub fn open(dir: &str) -> Result<Self, TantivyError> {
        let index = Index::open_in_dir(dir)?;
        println!("Opened `{dir}` index");

        let (searcher, query_parser) = SearchIndex::create_searcher(&index);

        Ok(Self {
            dir: dir.to_string(),
            index,
            skip_snippet: false,
            return_body: false,
            searcher: Some(searcher),
            query_parser: Some(query_parser),
            index_writer: None,
        })
    }

    fn create_schema() -> Schema {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("url", TEXT | STORED);
        schema_builder.add_text_field("body", TEXT | STORED);

        schema_builder.build()
    }

    fn create_searcher(index: &Index) -> (Searcher, QueryParser) {
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .unwrap();

        let searcher = reader.searcher();

        let mut query_parser = QueryParser::for_index(
            index,
            vec![
                index.schema().get_field("url").unwrap(),
                index.schema().get_field("title").unwrap(),
                index.schema().get_field("body").unwrap(),
            ],
        );

        query_parser.set_conjunction_by_default();

        (searcher, query_parser)
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

        let (searcher, query_parser) = SearchIndex::create_searcher(&self.index);

        self.searcher = Some(searcher);
        self.query_parser = Some(query_parser);

        Ok(0)
    }

    pub fn set_skip_snippet(&mut self) {
        self.skip_snippet = true;
    }

    pub fn set_return_body(&mut self) {
        self.return_body = true;
    }

    pub fn search(&self, query: &str) -> Result<Vec<SearchResult>, TantivyError> {
        let mut results = Vec::new();

        if let Some(query_parser) = &self.query_parser {
            if let Ok(query) = query_parser.parse_query(query) {
                if let Some(searcher) = &self.searcher {
                    if let Ok(docs) = searcher.search(&query, &TopDocs::with_limit(50)) {
                        if let Ok(mut snippet_generator) =
                            SnippetGenerator::create(searcher, &query, self.body())
                        {
                            snippet_generator.set_max_num_chars(80);

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

                                    let body = if self.return_body {
                                        Some(
                                            retrieved_doc
                                                .get_first(self.body())
                                                .unwrap()
                                                .as_text()
                                                .unwrap()
                                                .to_string(),
                                        )
                                    } else {
                                        None
                                    };

                                    let snippet = if !self.skip_snippet {
                                        Some(
                                            snippet_generator
                                                .snippet_from_doc(&retrieved_doc)
                                                .to_html()
                                                .replace('\n', " ")
                                                .trim()
                                                .to_string(),
                                        )
                                    } else {
                                        None
                                    };

                                    results.push(SearchResult {
                                        title,
                                        url,
                                        snippet,
                                        body,
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

    pub fn fuzzy_search_title(&self, query: &str) -> Result<Vec<SearchResult>, TantivyError> {
        self.fuzzy_search(query, &self.title())
    }

    pub fn fuzzy_search_body(&self, query: &str) -> Result<Vec<SearchResult>, TantivyError> {
        self.fuzzy_search(query, &self.body())
    }

    fn fuzzy_search(&self, query: &str, field: &Field) -> Result<Vec<SearchResult>, TantivyError> {
        let mut results = Vec::new();

        let term = Term::from_field_text(*field, query);
        let query = FuzzyTermQuery::new(term, 1, true);

        if let Some(searcher) = &self.searcher {
            if let Ok((docs, _count)) = searcher.search(&query, &(TopDocs::with_limit(10), Count)) {
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

                        let body = if self.return_body {
                            Some(
                                retrieved_doc
                                    .get_first(self.body())
                                    .unwrap()
                                    .as_text()
                                    .unwrap()
                                    .to_string(),
                            )
                        } else {
                            None
                        };

                        results.push(SearchResult {
                            title,
                            url,
                            body,
                            ..Default::default()
                        });
                    }
                }
            }
        }

        Ok(results)
    }
}
