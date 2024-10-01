mod app;
mod current;
mod error;
mod render;

pub use app::Application;
pub use current::{
    current, CurrentHooks, CurrentLanguage, CurrentLanguageState, CurrentSession,
    CurrentSessionState, CurrentState, CurrentThemeState, CurrentUser, CurrentUserState,
};
pub use error::{OrNotFound, Re, ReFuture, ServeError, ServeResult};
pub use render::Renderer;

pub async fn serve(router: axum::Router) -> anyhow::Result<()> {
    let addr = &crate::stuff::STUFF.web.addr;
    let listener = tokio::net::TcpListener::bind(addr.as_ref()).await?;

    tracing::info!(?addr, "serving");
    axum::serve(listener, router).await?;

    Ok(())
}
