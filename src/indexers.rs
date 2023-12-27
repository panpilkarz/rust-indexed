use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, Schema, STORED, TEXT};
use tantivy::{doc, Index, IndexWriter, ReloadPolicy, Searcher, SnippetGenerator, TantivyError};

pub struct SearchIndex {
    dir: String,
    pub index: Index,
    pub index_writer: IndexWriter,
    // schema
    pub title: Field,
    pub url: Field,
    pub body: Field,
    searcher: Option<Searcher>,
    query_parser: Option<QueryParser>,
}

#[derive(Debug)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub body: String,
    pub snippet: String,
}

impl SearchIndex {
    pub fn new(dir: &str) -> Result<Self, TantivyError> {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("url", TEXT | STORED);
        schema_builder.add_text_field("body", TEXT | STORED);

        let schema = schema_builder.build();
        let index = Index::create_in_dir(dir, schema.clone())?;
        let index_writer: IndexWriter = index.writer(50_000_000)?;

        println!("Created `{dir}` directory");

        let title = schema.get_field("title").unwrap();
        let url = schema.get_field("url").unwrap();
        let body = schema.get_field("body").unwrap();

        Ok(Self {
            dir: dir.to_string(),
            searcher: None,
            query_parser: None,
            index,
            index_writer,
            title,
            body,
            url,
        })
    }

    pub fn add_document(
        &mut self,
        title: String,
        url: String,
        body: String,
    ) -> Result<u64, TantivyError> {
        self.index_writer.add_document(doc!(
            self.title => title,
            self.url => url,
            self.body => body,
        ))
    }

    pub fn commit(&mut self) -> Result<u64, TantivyError> {
        self.index_writer.commit()?;

        let reader = self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()?;

        self.searcher = Some(reader.searcher());
        self.query_parser = Some(QueryParser::for_index(
            &self.index,
            vec![self.title, self.body],
        ));

        println!("Commited `{}` index", self.dir);
        Ok(0)
    }

    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let mut results = Vec::new();

        if let Some(query_parser) = &self.query_parser {
            if let Ok(query) = query_parser.parse_query(query) {
                if let Some(searcher) = &self.searcher {
                    if let Ok(docs) = searcher.search(&query, &TopDocs::with_limit(10)) {
                        if let Ok(snippet_generator) =
                            SnippetGenerator::create(searcher, &query, self.body)
                        {
                            println!("docs={}", docs.len());
                            for (_score, doc_address) in docs {
                                if let Ok(retrieved_doc) = searcher.doc(doc_address) {
                                    let title = retrieved_doc
                                        .get_first(self.title)
                                        .unwrap()
                                        .as_text()
                                        .unwrap()
                                        .to_string();
                                    let url = retrieved_doc
                                        .get_first(self.url)
                                        .unwrap()
                                        .as_text()
                                        .unwrap()
                                        .to_string();
                                    let body = retrieved_doc    // FIXME: don't return always
                                            .get_first(self.body)
                                            .unwrap()
                                            .as_text()
                                            .unwrap()
                                            .to_string();
                                    let snippet = snippet_generator // FIXME don't return always
                                        .snippet_from_doc(&retrieved_doc)
                                        .to_html()
                                        .replace('\n', " ")
                                        .trim()
                                        .to_string();

                                    results.push(SearchResult {
                                        title,
                                        url,
                                        body: String::new(), // FIXME
                                        snippet,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        results
    }
}
