use max_api_kernel::filters::{
    ChatFilter, FileFilter, MediaFilter, RegexTextFilter, SenderFilter, StatusFilter, TextFilter,
};
use max_api_kernel::{Filters, Message, MessageFilter, MessageStatus};

fn make_msg(text: &str, chat_id: Option<i64>, sender: Option<i64>) -> Message {
    Message {
        text: text.to_string(),
        chat_id,
        sender,
        ..Default::default()
    }
}

// --- Individual filters ---

#[test]
fn chat_filter_matches_exact_id() {
    let f = ChatFilter::new(42);
    assert!(f.check(&make_msg("", Some(42), None)));
    assert!(!f.check(&make_msg("", Some(43), None)));
    assert!(!f.check(&make_msg("", None, None)));
}

#[test]
fn text_filter_substring_match() {
    let f = TextFilter::new("hello".to_string());
    assert!(f.check(&make_msg("say hello there", None, None)));
    assert!(!f.check(&make_msg("goodbye", None, None)));
    assert!(!f.check(&make_msg("HELLO", None, None))); // case-sensitive
}

#[test]
fn sender_filter() {
    let f = SenderFilter::new(7);
    assert!(f.check(&make_msg("", None, Some(7))));
    assert!(!f.check(&make_msg("", None, Some(8))));
    assert!(!f.check(&make_msg("", None, None)));
}

#[test]
fn status_filter_edited() {
    let f = StatusFilter::new(MessageStatus::Edited);
    let mut msg = make_msg("", None, None);
    assert!(!f.check(&msg));
    msg.status = Some(MessageStatus::Edited);
    assert!(f.check(&msg));
    msg.status = Some(MessageStatus::Removed);
    assert!(!f.check(&msg));
}

#[test]
fn media_filter_empty_attaches() {
    let f = MediaFilter;
    let msg = make_msg("text only", None, None);
    assert!(!f.check(&msg));
}

#[test]
fn file_filter_no_attaches() {
    let f = FileFilter;
    assert!(!f.check(&make_msg("", None, None)));
}

#[test]
fn regex_filter_matches() {
    let f = RegexTextFilter::new(r"\d{4}").unwrap();
    assert!(f.check(&make_msg("code: 1234", None, None)));
    assert!(!f.check(&make_msg("no digits here", None, None)));
}

#[test]
fn regex_filter_invalid_pattern_errors() {
    assert!(RegexTextFilter::new(r"[invalid").is_err());
}

// --- Combinators (require concrete Sized types) ---

#[test]
fn and_filter_both_must_match() {
    let f = ChatFilter::new(1).and(TextFilter::new("hi".to_string()));
    assert!(f.check(&make_msg("hi there", Some(1), None)));
    assert!(!f.check(&make_msg("bye", Some(1), None)));
    assert!(!f.check(&make_msg("hi", Some(2), None)));
}

#[test]
fn or_filter_either_matches() {
    let f = ChatFilter::new(1).or(TextFilter::new("hi".to_string()));
    assert!(f.check(&make_msg("bye", Some(1), None))); // chat matches
    assert!(f.check(&make_msg("hi", Some(99), None))); // text matches
    assert!(!f.check(&make_msg("goodbye", Some(5), None)));
}

#[test]
fn not_filter_inverts() {
    let f = TextFilter::new("bad".to_string()).not();
    assert!(f.check(&make_msg("good message", None, None)));
    assert!(!f.check(&make_msg("bad content", None, None)));
}

#[test]
fn triple_chain() {
    let f = ChatFilter::new(10)
        .and(SenderFilter::new(20))
        .and(TextFilter::new("ok".to_string()));
    assert!(f.check(&make_msg("ok go", Some(10), Some(20))));
    assert!(!f.check(&make_msg("ok go", Some(10), Some(99))));
}

// --- Filters factory ---

#[test]
fn filters_chat_factory() {
    let f = Filters::chat(42);
    assert!(f.check(&make_msg("", Some(42), None)));
    assert!(!f.check(&make_msg("", Some(1), None)));
}

#[test]
fn filters_text_factory() {
    let f = Filters::text("world");
    assert!(f.check(&make_msg("hello world", None, None)));
    assert!(!f.check(&make_msg("nope", None, None)));
}

#[test]
fn filters_sender_factory() {
    let f = Filters::sender(5);
    assert!(f.check(&make_msg("", None, Some(5))));
    assert!(!f.check(&make_msg("", None, Some(6))));
}

#[test]
fn filters_status_factory() {
    let f = Filters::status(MessageStatus::Removed);
    let mut msg = make_msg("", None, None);
    assert!(!f.check(&msg));
    msg.status = Some(MessageStatus::Removed);
    assert!(f.check(&msg));
}

#[test]
fn filters_text_contains_factory() {
    let f = Filters::text_contains("needle");
    assert!(f.check(&make_msg("haystack needle found", None, None)));
    assert!(!f.check(&make_msg("nothing here", None, None)));
}

#[test]
fn filters_text_matches_factory() {
    let f = Filters::text_matches(r"^\d+$").unwrap();
    assert!(f.check(&make_msg("12345", None, None)));
    assert!(!f.check(&make_msg("abc", None, None)));
}

#[test]
fn filters_has_media_factory() {
    let f = Filters::has_media();
    assert!(!f.check(&make_msg("no media", None, None)));
}

#[test]
fn filters_has_file_factory() {
    let f = Filters::has_file();
    assert!(!f.check(&make_msg("no file", None, None)));
}
