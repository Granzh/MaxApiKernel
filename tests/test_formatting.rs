use max_api_kernel::formatting::Formatting;

fn parse(md: &str) -> (Vec<max_api_kernel::Element>, String) {
    Formatting::get_elements_from_markdown(md)
}

#[test]
fn plain_text_no_elements() {
    let (els, clean) = parse("hello world");
    assert_eq!(clean, "hello world");
    assert!(els.is_empty());
}

// Markup followed by more text — no trailing newline added.
#[test]
fn bold_in_middle() {
    let (els, clean) = parse("say **hello** now");
    assert_eq!(clean, "say hello now");
    assert_eq!(els.len(), 1);
    assert_eq!(els[0].type_, "STRONG");
    assert_eq!(els[0].from_, Some(4));
    assert_eq!(els[0].length, 5);
}

#[test]
fn bold_at_start_followed_by_text() {
    let (els, clean) = parse("**bold** text");
    assert_eq!(clean, "bold text");
    assert_eq!(els[0].type_, "STRONG");
    assert_eq!(els[0].from_, Some(0));
    assert_eq!(els[0].length, 4);
}

// Markup at end of string — the parser appends a '\n' to the clean text
// and adds 1 to the element length.
#[test]
fn italic_at_end_appends_newline() {
    let (els, clean) = parse("*italic*");
    assert_eq!(clean, "italic\n");
    assert_eq!(els.len(), 1);
    assert_eq!(els[0].type_, "EMPHASIZED");
    assert_eq!(els[0].from_, Some(0));
    assert_eq!(els[0].length, 7); // "italic" (6) + '\n' (1)
}

#[test]
fn underline_at_end_appends_newline() {
    let (els, clean) = parse("__under__");
    assert_eq!(clean, "under\n");
    assert_eq!(els.len(), 1);
    assert_eq!(els[0].type_, "UNDERLINE");
    assert_eq!(els[0].length, 6); // "under" (5) + '\n' (1)
}

#[test]
fn strikethrough_at_end_appends_newline() {
    let (els, clean) = parse("~~strike~~");
    assert_eq!(clean, "strike\n");
    assert_eq!(els.len(), 1);
    assert_eq!(els[0].type_, "STRIKETHROUGH");
    assert_eq!(els[0].length, 7); // "strike" (6) + '\n' (1)
}

#[test]
fn bold_at_end_appends_newline() {
    let (els, clean) = parse("text **bold**");
    assert_eq!(clean, "text bold\n");
    assert_eq!(els.len(), 1);
    assert_eq!(els[0].type_, "STRONG");
    assert_eq!(els[0].from_, Some(5));
    assert_eq!(els[0].length, 5); // "bold" (4) + '\n' (1)
}

// Two marks: first not at end (no newline), second at end (gets newline).
#[test]
fn two_marks_last_gets_newline() {
    let (els, clean) = parse("**a** and *b*");
    assert_eq!(clean, "a and b\n");
    assert_eq!(els.len(), 2);
    assert_eq!(els[0].type_, "STRONG");
    assert_eq!(els[0].length, 1);
    assert_eq!(els[1].type_, "EMPHASIZED");
    assert_eq!(els[1].length, 2); // "b" (1) + '\n' (1)
}

#[test]
fn leading_trailing_newlines_stripped() {
    let (_, clean) = parse("\nhello\n");
    assert_eq!(clean, "hello");
}

#[test]
fn empty_string() {
    let (els, clean) = parse("");
    assert_eq!(clean, "");
    assert!(els.is_empty());
}

#[test]
fn all_four_types_present() {
    let (els, _) = parse("**b** *i* __u__ ~~s~~");
    assert_eq!(els.len(), 4);
    let types: Vec<_> = els.iter().map(|e| e.type_.as_str()).collect();
    assert_eq!(
        types,
        ["STRONG", "EMPHASIZED", "UNDERLINE", "STRIKETHROUGH"]
    );
}

// Markup followed by '\n' also triggers has_newline — both behave the same.
#[test]
fn bold_followed_by_explicit_newline() {
    let (els, _) = parse("**word**\nrest");
    assert_eq!(els.len(), 1);
    // 'has_newline' was true (next byte is '\n'), so length includes the newline.
    assert_eq!(els[0].length, 5); // "word" (4) + '\n' (1)
}
