use aho_corasick::AhoCorasick;
use anyhow::Error;
use axum::response::{Html, IntoResponse, Response};
use html_escape::encode_text;
use reqwest::StatusCode;
use std::sync::OnceLock;

const HTML: &str = include_str!("fallback.html");
const PATTERNS_N: usize = 2;
const PATTERNS: [&str; PATTERNS_N] = ["{{original_error}}", "{{new_error}}"];

static AC: OnceLock<AhoCorasick> = OnceLock::new();

pub fn render(original_error: Error, new_error: Error) -> Response {
    let ac = AC.get_or_init(|| AhoCorasick::new(PATTERNS).unwrap());
    let fmt = |e| encode_text(&format!("{e:?}")).to_string();

    let original_error = fmt(original_error);
    let new_error = fmt(new_error);

    let values: &[&str; PATTERNS_N] = &[&original_error, &new_error];
    let html = ac.replace_all(HTML, values);

    (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response()
}
