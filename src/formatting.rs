// Copyright (c) 2026 FlintWithBlackCrown
// This file includes code derived from PyMax,
// Copyright (c) 2025 ink-developer, licensed under the MIT License.
// See the LICENSE file for details.

use crate::types::Element;
use regex::Regex;
use std::sync::OnceLock;

static MARKUP_PATTERN: OnceLock<Regex> = OnceLock::new();

fn markup_pattern() -> &'static Regex {
    MARKUP_PATTERN.get_or_init(|| {
        Regex::new(r"\*\*(?P<strong>.+?)\*\*|\*(?P<italic>.+?)\*|__(?P<underline>.+?)__|~~(?P<strike>.+?)~~")
            .expect("invalid markup regex")
    })
}

pub struct Formatting;

impl Formatting {
    pub fn get_elements_from_markdown(text: &str) -> (Vec<Element>, String) {
        let text = text.trim_matches('\n');
        let mut elements: Vec<Element> = Vec::new();
        let mut clean_parts: Vec<String> = Vec::new();
        let mut current_pos: i32 = 0;
        let mut last_end = 0;

        for mat in markup_pattern().find_iter(text) {
            let between = &text[last_end..mat.start()];
            if !between.is_empty() {
                clean_parts.push(between.to_string());
                current_pos += between.len() as i32;
            }

            let caps = markup_pattern().captures(mat.as_str());
            let (inner_text, fmt_type) = if let Some(caps) = caps {
                if let Some(m) = caps.name("strong") {
                    (Some(m.as_str()), Some("STRONG"))
                } else if let Some(m) = caps.name("italic") {
                    (Some(m.as_str()), Some("EMPHASIZED"))
                } else if let Some(m) = caps.name("underline") {
                    (Some(m.as_str()), Some("UNDERLINE"))
                } else if let Some(m) = caps.name("strike") {
                    (Some(m.as_str()), Some("STRIKETHROUGH"))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

            if let (Some(inner), Some(fmt)) = (inner_text, fmt_type) {
                let next_pos = mat.end();
                let has_newline = (next_pos < text.len()
                    && text.as_bytes().get(next_pos) == Some(&b'\n'))
                    || next_pos == text.len();

                let length = inner.len() as i32 + if has_newline { 1 } else { 0 };
                elements.push(Element {
                    type_: fmt.to_string(),
                    length,
                    from_: Some(current_pos),
                });

                clean_parts.push(inner.to_string());
                if has_newline {
                    clean_parts.push("\n".to_string());
                }

                current_pos += length;

                last_end = if has_newline && next_pos < text.len() {
                    mat.end() + 1
                } else {
                    mat.end()
                };
            } else {
                last_end = mat.end();
            }
        }

        let tail = &text[last_end..];
        if !tail.is_empty() {
            clean_parts.push(tail.to_string());
        }

        let clean_text = clean_parts.join("");
        (elements, clean_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bold() {
        let (elements, clean) = Formatting::get_elements_from_markdown("Hello **world**!");
        assert_eq!(clean, "Hello world!");
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].type_, "STRONG");
        assert_eq!(elements[0].length, 5);
    }

    #[test]
    fn test_no_markup() {
        let (elements, clean) = Formatting::get_elements_from_markdown("plain text");
        assert_eq!(clean, "plain text");
        assert!(elements.is_empty());
    }
}
