use crate::{reload::Reloader, stuff::STUFF};
use async_stream::try_stream;
use axum::{
    http::{header, HeaderName},
    response::sse::{Event, Sse},
};
use futures::Stream;
use std::convert::Infallible;

type Item = Result<Event, Infallible>;

const JS: &str = r#"let reload;

globalThis.onload = () => {
    reload = new EventSource("/__reload_sse__");

    reload.addEventListener("info", (event) => {
        console.log(event.data);
    });

    reload.addEventListener("reload", () => {
        location.reload();
    });
};

globalThis.onbeforeunload = () => {
    reload?.close();
};
"#;

pub async fn js() -> ([(HeaderName, &'static str); 1], &'static str) {
    let header = (header::CONTENT_TYPE, "text/javascript");
    let js = if STUFF.reload { JS } else { "" };
    ([header], js)
}

pub async fn sse(rl: Reloader) -> Sse<impl Stream<Item = Item>> {
    fn make_stream(rl: &Reloader) -> impl Stream<Item = Item> {
        let mut rx = rl.subscribe();
        try_stream! {
            if STUFF.reload {
                yield info_event("Reloading enabled!");
            }

            while let Ok(()) = rx.recv().await {
                yield reload_event();
            }
        }
    }

    let keep_alive = rl.keep_alive();
    let stream = make_stream(&rl);
    Sse::new(stream).keep_alive(keep_alive)
}

fn info_event(s: &str) -> Event {
    Event::default().event("info").data(s)
}

fn reload_event() -> Event {
    Event::default().event("reload").data("reload")
}
