use crate::{traits::Configurable, Result, config::Config};

/// run verify() then apply() on each stage, in order
pub fn run_pipeline(cfg: &Config, stages: Vec<Box<dyn Configurable>>) -> Result<()> {
    for stage in &stages {
        stage.verify(cfg)?;
    }
    for stage in stages {
        stage.apply(cfg)?;
    }
    Ok(())
}

