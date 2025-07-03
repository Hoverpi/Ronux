use crate::model::Config;
use anyhow::Result;

pub trait Configurable {
    /// Return Ok(true) if the ron file didn't have changes
    fn verify(&self, cfg: &Config) -> Result<bool>;
    /// Apply changes to reach the updated ron file
    fn apply(&self, cfg: &Config) -> Result<()>;
}
