use tantivy::schema::{Field, Schema, STORED, TEXT};
use tantivy::{doc, Index, IndexWriter, TantivyError};

pub struct SummaryMdIndex {
    pub index: Index,
    pub index_writer: IndexWriter,
    // schema
    pub title: Field,
    pub url: Field,
}

impl SummaryMdIndex {
    pub fn new(dir: &str) -> Result<Self, TantivyError> {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("url", TEXT | STORED);

        let schema = schema_builder.build();
        let index = Index::create_in_dir(dir, schema.clone())?;
        let index_writer: IndexWriter = index.writer(50_000_000)?;

        println!("Created `{dir}` directory for summary.md index");

        let title = schema.get_field("title").unwrap();
        let url = schema.get_field("url").unwrap();

        Ok(Self {
            index,
            index_writer,
            title,
            url,
        })
    }

    pub fn add_document(&mut self, title: String, url: String) -> Result<u64, TantivyError> {
        self.index_writer.add_document(doc!(
            self.title => title,
            self.url => url,
        ))
    }

    pub fn commit(&mut self) -> Result<u64, TantivyError> {
        println!("Commited summary.md index");
        self.index_writer.commit()
    }
}

pub struct PageIndex {
    pub index: Index,
    pub index_writer: IndexWriter,
    // schema
    pub title: Field,
    pub body: Field,
    pub url: Field,
}

impl PageIndex {
    pub fn new(dir: &str) -> Result<Self, TantivyError> {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("url", TEXT | STORED);
        schema_builder.add_text_field("body", TEXT | STORED);

        let schema = schema_builder.build();
        let index = Index::create_in_dir(dir, schema.clone())?;
        let index_writer: IndexWriter = index.writer(50_000_000)?;

        println!("Created `{dir}` directory for pages index");

        let title = schema.get_field("title").unwrap();
        let url = schema.get_field("url").unwrap();
        let body = schema.get_field("body").unwrap();

        Ok(Self {
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
        println!("Commited page index");
        self.index_writer.commit()
    }
}
