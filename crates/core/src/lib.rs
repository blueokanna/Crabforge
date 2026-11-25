pub mod agents;
mod anthropic_auth;
mod bedrock;
pub mod catalog;
pub mod config;
pub mod gateway;
mod media;
mod model_name;
mod opencode;
pub mod peer;
pub mod service;
mod translate;
mod vertex;
pub mod webchat;

pub const APP_ID: &str = "com.blue.crabforge";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
