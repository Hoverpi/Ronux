use std::{fs, path::PathBuf, path::Path};
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use ron::{Options, extensions::Extensions};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(skip)] pub system_config: PathBuf,

    /// System base installation config
    pub firmware: Firmware,
    pub disks: Vec<Disk>,
    pub lvm: Option<Lvm>,

    /// Bootloader config
    pub bootloader: Bootloader,

    /// Kernel config
    pub kernel: Kernel,

    /// Installed packages
    #[serde(default)] pub packages: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum Firmware {
    BIOS,
    UEFI,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Disk {
    pub path: String,
    pub table: String,
    pub partitions: Vec<Partition>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Partition {
    pub name: String,
    pub size_gb: Option<f32>,
    pub fs: Option<String>,
    #[serde(default)] pub flags: Option<Vec<String>>,
    #[serde(default)] pub luks: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Lvm {
    pub vg: String,
    pub volumes: Vec<LogicalVolume>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LogicalVolume {
    pub name: String,
    pub size_gb: Option<f32>,
    pub fs: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Bootloader {
    #[serde(rename = "type")] pub type_: String,
    #[serde(default)] pub theme: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Kernel {
    pub name: String,
    pub initramfs_hooks: Vec<String>,
    #[serde(default)] pub parameters: Option<Vec<String>>,
    pub firmware: bool,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_owned();
        let text = fs::read_to_string(&path)
            .with_context(|| format!("Could not be read {}", path.display()))?;

        // Make it more readible
        let mut opts = Options::default();
        opts.default_extensions.insert(Extensions::IMPLICIT_SOME);

        // Parse the ron file
        let mut config: Config = opts.from_str(&text)
            .context("Error parsing RON file")?;

        config.system_config = path;
        Ok(config)
    }
}
