use std::collections::HashSet;

use crate::index::{SearchIndex, SearchResult};
use crate::{INDEX_CODE_DIR, INDEX_PAGE_DIR};

pub struct Ranking {
    /// Pages
    index_page: SearchIndex,
    /// Code blocks
    index_code: SearchIndex,
}

impl Default for Ranking {
    fn default() -> Self {
        Self::new()
    }
}

impl Ranking {
    pub fn new() -> Self {
        let index_page = SearchIndex::open(INDEX_PAGE_DIR).unwrap();
        let mut index_code = SearchIndex::open(INDEX_CODE_DIR).unwrap();

        // This index has code in the body, we want to return it, without snippet.
        index_code.set_return_body();
        index_code.set_skip_snippet();

        Self {
            index_page,
            index_code,
        }
    }

    pub fn search(&self, q: &str) -> Vec<SearchResult> {
        let mut urls = HashSet::<String>::new();
        let mut results = Vec::<SearchResult>::new();

        // 1/3 search full query '"impl trait"'
        let all_words_q = format!("\"{}\"", q);

        for index in [&self.index_page, &self.index_code] {
            if let Ok(res) = index.search(&all_words_q) {
                results.extend(res);
            }
        }

        results.iter().for_each(|r| {
            urls.insert(r.url.clone());
        });

        // 2/3 if no results, search 'impl trait'
        for index in [&self.index_page, &self.index_code] {
            if let Ok(res) = index.search(q) {
                let res: Vec<SearchResult> = res
                    .into_iter()
                    .filter(|r| urls.get(&r.url).is_none())
                    .collect();
                results.extend(res);
            }
        }

        results.iter().for_each(|r| {
            urls.insert(r.url.clone());
        });

        // 3/3 if no results, fuzzy search in title and then in body
        if results.is_empty() {
            for index in [&self.index_page, &self.index_code] {
                if let Ok(res) = index.fuzzy_search_title(q) {
                    let res: Vec<SearchResult> = res
                        .into_iter()
                        .filter(|r| urls.get(&r.url).is_none())
                        .collect();
                    results.extend(res);
                }
            }

            results.iter().for_each(|r| {
                urls.insert(r.url.clone());
            });

            for index in [&self.index_page, &self.index_code] {
                if let Ok(res) = index.fuzzy_search_body(q) {
                    let res: Vec<SearchResult> = res
                        .into_iter()
                        .filter(|r| urls.get(&r.url).is_none())
                        .collect();
                    results.extend(res);
                }
            }
        }

        results
    }

    // fn autocomplete(q: &str) {
    //     todo!();
    // }
}
