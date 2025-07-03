use anyhow::{Result, Context};
use serde::Deserialize;
use crate::CommandRunner;

pub struct Lsblk;

#[derive(Debug, Deserialize)]
pub struct LsblkOutput {
    pub blockdevices: Vec<BlockDevice>,
}

#[derive(Debug, Deserialize)]
pub struct BlockDevice {
    pub name: String,         // same as KNAME
    #[serde(rename = "kname")]
    pub kname: String,        // kernel name, e.g. "nvme0n1"
    #[serde(default)]
    pub path: Option<String>, // full path, e.g. "/dev/nvme0n1"
    #[serde(default)]
    pub mountpoints: Vec<Option<String>>,
    #[serde(default)]
    pub children: Vec<BlockDevice>,
}

impl Lsblk {
    pub fn load<P: Into<String>>(path: P) -> Result<LsblkOutput> {
        let path = path.into();
        let output = CommandRunner::new("lsblk")
            .arg(&path)
            .arg("-o").arg("NAME,KNAME,PATH,MOUNTPOINTS")
            .arg("-n")
            .arg("-J")
            .capture()
            .run()?;

        let parsed: LsblkOutput = serde_json::from_str(&output)
            .context("failed to deserialize lsblk JSON")?;
        Ok(parsed)
    }
}
