use anyhow::{Result, Context};
use core::{Configurable, Config, CommandRunner, Lsblk};

pub struct CryptoManager;

impl Configurable for CryptoManager {
    fn verify(&self, cfg: &Config) -> Result<bool> {
        // Loop through disks with luks=true partitions
        for disk in &cfg.disks {
            let tree = Lsblk::load(&disk.path)
                .with_context(|| format!("lsblk on {}", disk.path))?;

            let parent = tree.blockdevices.into_iter()
                .find(|d| d.path.as_deref() == Some(&disk.path))
                .context(format!("disk {} not found", disk.path))?;

            for part in &disk.partitions {
                if part.luks.unwrap_or(false) {
                    let child = parent.children.iter()
                        .find(|c| c.name.ends_with(&part.name))
                        .with_context(|| format!("LUKS partition '{}' not found under {}", part.name, disk.path))?;
                    // If this partition already shows as encrypted, skip
                    if child.kname.starts_with("dm-") {
                        continue;
                    } else {
                        return Ok(true); // needs luksFormat + luksOpen
                    }
                }
            }
        }
        Ok(false)
    }

    fn apply(&self, cfg: &Config) -> Result<()> {
        for disk in &cfg.disks {
            let tree = Lsblk::load(&disk.path)?;
            let parent = tree.blockdevices.into_iter()
                .find(|d| d.path.as_deref() == Some(&disk.path))
                .unwrap();

            for part in &disk.partitions {
                if part.luks.unwrap_or(false) {
                    // find raw partition path
                    let child = parent.children.iter()
                        .find(|c| c.name.ends_with(&part.name))
                        .unwrap();
                    let dev = child.path.as_ref().unwrap();

                    // Format as LUKS
                    CommandRunner::new("cryptsetup")
                        .arg("luksFormat")
                        .arg(dev)
                        .arg("--type").arg("luks2")
                        .sudo()
                        .run()
                        .with_context(|| format!("luksFormat on {}", dev))?;

                    // Open the device
                    let mapper = part.name.clone(); // e.g., "cryptlvm"
                    CommandRunner::new("cryptsetup")
                        .arg("luksOpen")
                        .arg(dev)
                        .arg(&mapper)
                        .sudo()
                        .run()
                        .with_context(|| format!("luksOpen {} → {}", dev, mapper))?;
                }
            }
        }
        Ok(())
    }
}