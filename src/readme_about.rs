//! Extracts the README's About section. The build script does this and the application compiles
//! in the result. It is a module of the application only so that `cargo test` runs its tests,
//! which it does not do for a build script.

/// The text of the README's About section, leaving out HTML such as the screenshot. Each
/// paragraph is on one line, as the lines it is wrapped to in the file are not part of the text.
pub fn about_text(readme: &str) -> String {
    let lines: Vec<&str> = readme
        .lines()
        .skip_while(|line| line.trim() != "## About")
        .skip(1)
        .take_while(|line| !line.starts_with("## "))
        .filter(|line| !line.trim_start().starts_with('<'))
        .map(str::trim)
        .collect();
    let paragraphs: Vec<String> = lines
        .join("\n")
        .split("\n\n")
        .map(|paragraph| paragraph.trim().replace('\n', " "))
        .filter(|paragraph| !paragraph.is_empty())
        .collect();
    paragraphs.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_stops_at_the_next_section_and_keeps_paragraphs() {
        let readme = "# Name\n\n## About\r\n\r\nOne\r\nline.\r\n\r\n<img src=\"a.png\">\r\n\r\nTwo.\r\n\r\n## Next\r\n\r\nNot this.\r\n";

        assert_eq!(about_text(readme), "One line.\n\nTwo.");
    }

    #[test]
    fn it_is_empty_without_the_section() {
        assert_eq!(about_text("# No such section"), "");
    }
}
