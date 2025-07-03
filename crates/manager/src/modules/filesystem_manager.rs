use anyhow::{Result, Context};
use core::{Configurable, Config, CommandRunner, Lsblk};

pub struct FileSystemManager;

impl Configurable for FileSystemManager {
    fn verify(&self, cfg: &Config) -> Result<bool> {
        
    }

    fn apply(&self, cfg: &Config) -> Result<()> {
        
    }
}