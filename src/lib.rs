#![allow(async_fn_in_trait)]
#![warn(missing_debug_implementations)]

pub use anyhow as error;
pub use axum;
pub use axum::async_trait;
pub use liquid;
pub use tokio;
pub use tower;
pub use tower_cookies;
pub use tower_http;

pub mod binary;
pub mod db;
pub mod helper;
pub mod reload;
pub mod scratch;
pub mod scripts;
pub mod serve;
pub mod stuff;
pub mod styles;
pub mod themes;

pub use serve::serve;
