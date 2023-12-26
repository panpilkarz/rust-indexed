fn parse_link_descripton(s: &str) -> &str {
    if let Some(index) = s.find('[') {
        return &s[index + 1..];
    }
    s
}

pub fn parse_summary_md(s: &str) -> Vec<(String, String)> {
    s.split("\n")
        .map(|x| x.trim())
        .filter(|x| x.len() > 0)
        .map(|x| x.split("](").collect())
        .filter(|x: &Vec<&str>| x.len() == 2)
        .map(|x: Vec<&str>| {
            (
                parse_link_descripton(x[0]).to_string(),
                x[1].replace(".md)", ""),
            )
        })
        .filter(|x| x.0.len() > 0 && x.1.len() > 0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
