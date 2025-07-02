// declare modules
pub mod config;           // <– parser
pub mod traits;           // <– Configurable
pub mod orchestrator;     // <– run_pipeline()
pub mod stages;          // <– all the installers
pub mod utils;          // <- Centralize the commands

// re‑exports for easy use
pub use config::Config;
// pub use orchestrator::run_pipeline;
pub use traits::Configurable;

// unify error type
pub type Result<T> = anyhow::Result<T>;