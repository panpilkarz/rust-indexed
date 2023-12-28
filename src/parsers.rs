use std::path::PathBuf;

fn parse_link_descripton(s: &str) -> &str {
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
                return Some(buf);
            } else {
                eprintln!("Couldn't open {:?}", path);
            }
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
                parse_link_descripton(x[0]).to_string(),
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

    for line in s.split('\n') {
        // Code block start/end
        if line.starts_with("```") {
            // FIXME: ```rust
            if in_code {
                if !code.is_empty() {
                    code_blocks.push(code.trim().to_string());
                }
                code.clear();
                in_code = false;
                continue;
            }
            in_code = true;
            continue;
        }

        // Skip underlines
        if !in_code && (line.starts_with("---") || line.starts_with("___")) {
            continue;
        }

        // Handle includes
        if let Some(line_stripped) = line.strip_prefix("{{") {
            if let Some(buf) = parse_include(line_stripped, md_dir) {
                new_s.push_str(&buf);
                if in_code {
                    code.push_str(&buf);
                    code.push('\n');
                }
            }
            continue;
        }

        let line = line.trim_end();

        if in_code {
            code.push_str(line);
            code.push('\n');
        }

        new_s.push_str(line);
        new_s.push('\n');
    }

    (new_s.trim().to_string(), code_blocks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_link_description() {
        assert_eq!(parse_link_descripton("   [hello"), "hello");
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
```shell
loop {
    let a = String::new();
}
```
        ",
            ".",
        );

        assert_eq!(
            body,
            "# Example code 1
let x = 1;

# Example code 2
loop {
    let a = String::new();
}"
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
```
{{#include }}
```
```
{{#include src/main.rs}}
```
```
{{#include  src/parsers.rs  }}
```
```
{{#rustdoc_include src/indexers.rs}}
```
```
{{ #include src/indexers.rs }}
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
}
