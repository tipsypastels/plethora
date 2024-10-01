use plethora::{
    axum::{
        extract::FromRequestParts, http::request::Parts, middleware::from_fn_with_state,
        routing::get, Router,
    },
    db::{Db, Id},
    error::Result,
    serve::{current, Application, CurrentHooks, CurrentState, Renderer, ServeResult},
    styles::Styles,
    themes::{props, Themes, ThemesInit},
    tower::ServiceBuilder,
    tower_cookies::CookieManagerLayer,
};
use serde::Serialize;
use std::convert::Infallible;

#[tokio::main]
async fn main() -> Result<()> {
    let _guard = plethora::stuff::builder().file("stuff.toml", true).init()?;
    plethora::scratch::init().await?;
    plethora::binary::install().await?;

    let db = Db::new().await?;
    let styles = Styles::new().await?;
    let themes = Themes::new(ThemesInit {
        styles: styles.clone(),
    })
    .await?;

    let app = App { db, styles, themes };

    let cookies = CookieManagerLayer::new();
    let current = from_fn_with_state(app.clone(), current::<Current, App>);

    let router = Router::new()
        .route("/", get(index))
        .layer(ServiceBuilder::new().layer(cookies).layer(current))
        .with_state(app);

    plethora::serve(router).await
}

async fn index(re: Render) -> ServeResult {
    re.render("index", props!({}))
}

#[derive(Debug, Clone)]
struct App {
    pub db: Db,
    pub styles: Styles,
    pub themes: Themes,
}

impl Application for App {
    fn db(&self) -> &Db {
        &self.db
    }

    fn styles(&self) -> &Styles {
        &self.styles
    }

    fn themes(&self) -> &Themes {
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

impl CurrentHooks for Current {
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
struct Render {
    app: App,
    current: CurrentState<Current>,
}

impl Renderer for Render {
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
impl FromRequestParts<App> for Render {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, app: &App) -> Result<Self, Infallible> {
        let current = CurrentState::<Current>::from_request_parts(parts, app).await?;
        let app = app.clone();

        Ok(Self { app, current })
    }
}
