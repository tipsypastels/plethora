//! The format of esbuild logs is as follows.
//!
//! ```text
//! ✘ [ERROR] Could not resolve "@hoxtwired/stimulus" [lint]
//!
//!    scripts/packs/base.ts:3:28:
//!      3 │ import { Application } from "@hoxtwired/stimulus";
//!        ╵                             ~~~~~~~~~~~~~~~~~~~~~
//!
//!  You can mark the path "@hoxtwired/stimulus" as external to exclude it from the bundle, which will remove this error and leave the unresolved path in the bundle.
//! ```
//!
//! We want to output them as tracing-friendly messages, so we only need the
//! initial message and the location (as a field) for each entry.

use anyhow::Result;
use async_stream::try_stream;
use futures::{Stream, StreamExt};
use std::pin::pin;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

struct Entry {
    msg: String,
    src: String,
    lint: Option<String>,
    kind: Kind,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
enum Kind {
    Warning,
    Error,
}

const WARNING_CHAR: char = '▲';
const ERROR_CHAR: char = '✘';

pub async fn capture(reader: impl AsyncRead + Unpin) -> Result<()> {
    let mut stream = pin!(parse(reader));
    while let Some(result) = stream.next().await {
        const TARGET: &str = "forumm::esbuild";

        use Kind as K;
        let Entry {
            msg,
            src,
            kind,
            lint,
        } = result?;

        match (kind, lint) {
            (K::Error, Some(lint)) => {
                tracing::error!(target: TARGET, %src, %lint, "{msg}");
            }
            (K::Error, None) => {
                tracing::error!(target: TARGET, %src, "{msg}");
            }
            (K::Warning, Some(lint)) => {
                tracing::warn!(target: TARGET, %src, %lint, "{msg}");
            }
            (K::Warning, None) => {
                tracing::warn!(target: TARGET, %src, "{msg}");
            }
        }
    }
    Ok(())
}

fn parse(reader: impl AsyncRead + Unpin) -> impl Stream<Item = Result<Entry>> {
    let mut reader = BufReader::new(reader).lines();
    try_stream! {
        while let Some(line) = reader.next_line().await? {
            if line.starts_with(' ') {
                continue;
            }
            let Some((kind, msg, lint)) = parse_msg_line(&line) else {
                continue
            };
            let Some(_) = reader.next_line().await? else {
                continue; // skip blank
            };
            let Some(src) = reader.next_line().await? else {
                continue;
            };

            yield Entry {
                msg: msg.to_ascii_lowercase(),
                src: src.trim().to_string(),
                lint: lint.map(ToString::to_string),
                kind,
            }
        }
    }
}

fn parse_msg_line(line: &str) -> Option<(Kind, &str, Option<&str>)> {
    let kind = match line.chars().next() {
        Some(WARNING_CHAR) => Kind::Warning,
        Some(ERROR_CHAR) => Kind::Error,
        _ => return None,
    };

    let end_of_kind_index = line.find(']')? + 1; // +1 for space
    let line_after_kind = line.get(end_of_kind_index..)?;

    let start_of_lint_index = line_after_kind.find('[');
    let (msg, lint) = match start_of_lint_index {
        None => (line_after_kind.trim(), None),
        Some(idx) => {
            let (msg, lint_and_brackets) = line_after_kind.split_at(idx);
            let lint_and_close_bracket = lint_and_brackets.get(1..)?;
            let lint_close_bracket_index = lint_and_close_bracket.find(']')?;
            let lint = lint_and_close_bracket.get(..lint_close_bracket_index)?;

            (msg.trim(), Some(lint.trim()))
        }
    };
    Some((kind, msg, lint))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn msg_line() {
        use parse_msg_line as p;

        assert_eq!(
            p(r#"▲ [WARNING] Bad thing happened"#),
            Some((Kind::Warning, "Bad thing happened", None))
        );

        assert_eq!(
            p(r#"✘ [ERROR] Bad thing happened [lint]"#),
            Some((Kind::Error, "Bad thing happened", Some("lint")))
        );
    }
}
