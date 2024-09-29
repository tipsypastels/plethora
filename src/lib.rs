#![allow(async_fn_in_trait)]
#![warn(missing_debug_implementations)]

pub use anyhow as error;

pub mod binary;
pub mod db;
pub mod scratch;
pub mod server;
pub mod stuff;
pub mod styles;
pub mod themes;
