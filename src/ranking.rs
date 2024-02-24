use std::collections::HashSet;

use crate::index::{SearchIndex, SearchResult};
use crate::{INDEX_CODE_DIR, INDEX_PAGE_DIR};
use bitflags::bitflags;

bitflags! {
    pub struct SearchFlags: u32 {
        const DEFAULT   = 0b00000001;
        const CODE_ONLY = 0b00000010;
    }
}

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

    pub fn search(&self, q: &str, flags: SearchFlags) -> Vec<SearchResult> {
        let mut urls = HashSet::<String>::new();
        let mut results = Vec::<SearchResult>::new();
        let mut prev_len: usize;

        let indexes = match flags.contains(SearchFlags::CODE_ONLY) {
            true => vec![&self.index_code],
            false => vec![&self.index_page, &self.index_code],
        };

        // 1/3 search full query '"impl trait"'
        let all_words_q = format!("\"{}\"", q);

        for index in &indexes {
            if let Ok(res) = index.search(&all_words_q) {
                results.extend(res);
            }
        }

        results.iter().for_each(|r| {
            urls.insert(r.url.clone());
        });
        prev_len = results.len();

        // 2/3 if no results, search 'impl trait'
        for index in &indexes {
            if let Ok(res) = index.search(q) {
                let res: Vec<SearchResult> = res
                    .into_iter()
                    .filter(|r| urls.get(&r.url).is_none())
                    .collect();
                results.extend(res);
            }
        }

        results.iter().skip(prev_len).for_each(|r| {
            urls.insert(r.url.clone());
        });

        // 3/3 if no results, fuzzy search in title and then in body
        if results.is_empty() {
            prev_len = results.len();

            for index in &indexes {
                if let Ok(res) = index.fuzzy_search_title(q) {
                    let res: Vec<SearchResult> = res
                        .into_iter()
                        .filter(|r| urls.get(&r.url).is_none())
                        .collect();
                    results.extend(res);
                }
            }

            results.iter().skip(prev_len).for_each(|r| {
                urls.insert(r.url.clone());
            });

            for index in &indexes {
                if let Ok(res) = index.fuzzy_search_body(q) {
                    let res: Vec<SearchResult> = res
                        .into_iter()
                        .filter(|r| urls.get(&r.url).is_none())
                        .collect();
                    results.extend(res);
                }
            }
        }

        for tok in q.split(' ') {
            let highligthed = format!("<b>{}</b>", tok);
            results.iter_mut().for_each(|r| {
                if r.body.is_some() {
                    r.body = Some(
                        String::from(r.body.as_ref().unwrap()).replace(tok, highligthed.as_str()),
                    )
                }
            });
        }

        results
    }

    // fn autocomplete(q: &str) {
    //     todo!();
    // }
}
