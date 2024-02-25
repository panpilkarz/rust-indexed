use std::path::PathBuf;

use html_escape::decode_html_entities;
use lazy_static::lazy_static;
use regex::Regex;

fn strip_tags(html: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"<[^>]*>").unwrap();
    }
    RE.replace_all(html, "").to_string()
}

fn strip_refs(text: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\[[^\]]*\]").unwrap();
    }
    RE.replace_all(text, "").to_string()
}

fn strip_ticks(text: &str) -> String {
    text.chars().filter(|&ch| ch != '`').collect()
}

fn parse_html_title(html: &str) -> Option<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"<title>(.*?)</title>").unwrap();
    }
    if let Some(captures) = RE.captures(html) {
        if let Some(title) = captures.get(1) {
            return Some(title.as_str().to_string());
        }
    }
    None
}

fn seek_link_description(s: &str) -> &str {
    if let Some(index) = s.find('[') {
        return &s[index + 1..];
    }
    s
}

fn parse_include(s: &str, md_dir: &str) -> Option<String> {
    let toks: Vec<_> = s
        .split(' ')
        .filter(|&x| x == "#include" || x == "#rustdoc_include")
        .collect();

    if !toks.is_empty() {
        let filename = s.replace("{{", "").replace("}}", "").replace(toks[0], "");

        let filename = filename.trim();

        if !filename.is_empty() {
            // ../src/main.rs:anchor
            let path = PathBuf::from(md_dir).join(filename.split(':').next().unwrap());
            if let Ok(buf) = std::fs::read_to_string(&path) {
                let code: Vec<_> = buf
                    .lines()
                    .filter(|&line| !(line.starts_with("// ") || line.eq("//")))
                    .collect();
                return Some(code.join("\n"));
            }

            eprintln!("Couldn't open {:?}", path);
        }
    }
    None
}

pub fn parse_summary_md(s: &str) -> Vec<(String, String)> {
    s.split('\n')
        .map(|x| x.trim())
        .filter(|x| !x.is_empty())
        .map(|x| x.split("](").collect())
        .filter(|x: &Vec<&str>| x.len() == 2)
        .map(|x: Vec<&str>| {
            (
                seek_link_description(x[0]).to_string(),
                x[1].replace(".md)", ""),
            )
        })
        .filter(|x| !(x.0.is_empty() || x.1.is_empty()))
        .collect()
}

pub fn parse_md_page(s: &str, md_dir: &str) -> (String, Vec<String>) {
    let mut new_s = String::new();
    let mut code_blocks: Vec<String> = vec![];
    let mut code = String::new();
    let mut in_code = false;

    // Handle includes
    for line in s.split('\n') {
        if let Some(line_stripped) = line.strip_prefix("{{") {
            if let Some(buf) = parse_include(line_stripped, md_dir) {
                new_s.push_str(&buf);
                new_s.push('\n');
            }
            continue;
        }

        new_s.push_str(line);
        new_s.push('\n');
    }

    let s = new_s;
    let mut new_s = String::new();

    for line in s.split('\n') {
        // Code block start/end
        if line.starts_with("```") {
            if in_code {
                if !code.is_empty() {
                    code_blocks.push(code.trim().to_string());
                }
                code.clear();
                in_code = false;
                continue;
            }
            if line.starts_with("```rust") {
                in_code = true;
            }
            continue;
        }

        // Trim the end but not the beginning to keep indentation
        let line = line.trim_end();

        if in_code {
            code.push_str(line);
            code.push('\n');
            continue;
        }

        // Skip underlines
        if line.starts_with("---") || line.starts_with("___") || line.starts_with('[') {
            continue;
        }

        // Trim leading special markup chars
        let line = line.trim_start_matches('#').trim_start();
        let line = line.trim_start_matches('*').trim_start();

        // Strip "decorators"
        let line = &strip_tags(line);
        let line = &strip_refs(line);
        let line = &strip_ticks(line);

        new_s.push_str(line);
        new_s.push('\n');
    }

    (new_s.trim().to_string(), code_blocks)
}

pub fn parse_html_page(html: &str) -> (String, Vec<String>, Option<String>) {
    // todo: parse code blocks
    let title = parse_html_title(html);

    let text = strip_tags(html);
    let text = decode_html_entities(&text).to_string();

    (text, vec![], title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seek_link_description() {
        assert_eq!(seek_link_description("   [hello"), "hello");
    }

    #[test]
    fn test_parse_summary_md() {
        let parsed = parse_summary_md(
            "
            [Welcome to Comprehensive Rust 🦀](index.md)
           
            - [Running the Course](running-the-course.md)
            - [Course Structure](running-the-course/course-structure.md)
        
            - [Keyboard Shortcuts](running-the-course/keyboard-shortcuts.md)
            - [Translations](running-the-course/translations.md)",
        );

        assert!(parsed.len() == 5);
        assert!(parsed[0].0.as_str() == "Welcome to Comprehensive Rust 🦀");
        assert!(parsed[0].1.as_str() == "index");

        assert!(parsed[2].0.as_str() == "Course Structure");
        assert!(parsed[2].1.as_str() == "running-the-course/course-structure");
    }

    #[test]
    fn test_parse_page_with_code() {
        let (body, code_blocks) = parse_md_page(
            "
# Example code 1
```rust,ignore
let x = 1;
```

# Example code 2
```rust
loop {
    let a = String::new();
}
```
        ",
            ".",
        );

        assert_eq!(
            body,
            "Example code 1

Example code 2"
        );

        assert_eq!(code_blocks[0], "let x = 1;");
        assert_eq!(
            code_blocks[1],
            "loop {
    let a = String::new();
}"
        );
    }

    #[test]
    fn test_parse_page_with_include() {
        let (body, code_blocks) = parse_md_page(
            "
```nok
{{#include }}
```
```rust
{{#include }}
```
```rust ok
{{#include src/main.rs}}
```
```rust ok
{{#include src/lib.rs  }}
```
```rust ok
{{#rustdoc_include src/index.rs}}
```
```rust ok
{{ #include src/index.rs }}
```
```shell nok
{{ #include src/index.rs }}
```
        ",
            ".",
        );

        assert!(body.len() > 1024);
        assert!(code_blocks.len() == 4);
    }

    #[test]
    fn test_parse_path_with_underlines() {
        let (body, _) = parse_md_page(
            "
---
NOTES
___
        ",
            "/tmp/",
        );

        assert_eq!(body, "NOTES");
    }

    #[test]
    fn test_ignore_header_hashes() {
        let (body, _) = parse_md_page(
            "
# Head
content
## Head2
content2
            ",
            "",
        );

        assert_eq!(
            body,
            "Head
content
Head2
content2"
        );
    }
}
