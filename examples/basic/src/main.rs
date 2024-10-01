use plethora::{
    axum::{
        extract::FromRequestParts, http::request::Parts, middleware::from_fn_with_state,
        routing::get, Router,
    },
    db::{Db, Id},
    error::Result,
    serve::{layer, AsApp, CurrentState, Render, ServeResult},
    stuff::STUFF,
    styles::Styles,
    themes::{props, Themes},
    tower::ServiceBuilder,
    tower_cookies::CookieManagerLayer,
};
use serde::Serialize;
use std::convert::Infallible;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    let _guard = plethora::stuff::builder().file("stuff.toml", true).init()?;
    plethora::scratch::init().await?;
    plethora::binary::install().await?;

    let db = Db::new().await?;
    let styles = Styles::new().await?;
    let themes = Themes::new(styles.clone()).await?;
    let app = App { db, styles, themes };

    let addr = &STUFF.web.addr;
    let listener = TcpListener::bind(addr.as_ref()).await?;

    let cookies = CookieManagerLayer::new();
    let current = from_fn_with_state(app.clone(), layer::<App, Current>);

    let router = Router::new()
        .route("/", get(index))
        .layer(ServiceBuilder::new().layer(cookies).layer(current))
        .with_state(app);

    plethora::axum::serve(listener, router).await?;
    Ok(())
}

async fn index(re: Renderer) -> ServeResult {
    re.render("index", props!({}))
}

#[derive(Debug, Clone)]
struct App {
    pub db: Db,
    pub styles: Styles,
    pub themes: Themes,
}

impl AsApp for App {
    fn as_db(&self) -> &Db {
        &self.db
    }

    fn as_styles(&self) -> &Styles {
        &self.styles
    }

    fn as_themes(&self) -> &Themes {
        &self.themes
    }
}

#[plethora::async_trait]
impl FromRequestParts<App> for App {
    type Rejection = Infallible;

    async fn from_request_parts(_: &mut Parts, app: &App) -> Result<Self, Infallible> {
        Ok(app.clone())
    }
}

#[derive(Debug, Clone)]
struct Current;

impl plethora::serve::Current for Current {
    type Session = Session;
    type User = User;

    async fn session(_db: &Db, _id: Id) -> Result<Option<Self::Session>> {
        Ok(Some(Session {
            user_id: Id::new_v4(),
        }))
    }

    async fn user(_db: &Db, _id: Id) -> Result<Option<Self::User>> {
        Ok(Some(User))
    }

    fn user_id(session: &Self::Session) -> Id {
        session.user_id
    }
}

#[derive(Debug, Clone, Serialize)]
struct Session {
    user_id: Id,
}
#[derive(Debug, Clone, Serialize)]
struct User;

#[derive(Debug, Clone)]
struct Renderer {
    app: App,
    current: CurrentState<Current>,
}

impl Render for Renderer {
    type App = App;
    type Current = Current;

    fn app(&self) -> &Self::App {
        &self.app
    }

    fn current(&self) -> &CurrentState<Self::Current> {
        &self.current
    }
}

#[plethora::async_trait]
impl FromRequestParts<App> for Renderer {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, app: &App) -> Result<Self, Infallible> {
        let current = CurrentState::<Current>::from_request_parts(parts, app).await?;
        let app = app.clone();

        Ok(Self { app, current })
    }
}
