use core::{CommandRunner, Config, Configurable, Lsblk, Partition, BlockDevice};
use anyhow::{Result, Context};

pub struct PartitionManager;

impl PartitionManager {
    /// Compare RON partitions vs. actual children
    fn compare_partitions(expected: &[Partition], actual: &[BlockDevice]) -> bool {
        expected.iter().all(|part| {
            // find a matching BlockDevice by name suffix
            if let Some(dev) = actual.iter().find(|d| d.name.ends_with(&part.name)) {
                // check encryption child presence
                if part.luks.unwrap_or(false) {
                    !dev.children.is_empty()
                } else {
                    true
                }
            } else {
                false
            }
        })
    }
}

impl Configurable for PartitionManager {
    fn verify(&self, cfg: &Config) -> Result<bool> {
        

        for disk in cfg.disks.iter() {
            // Load tree of the system
            let tree = Lsblk::load(disk.path.as_str())
                .with_context(|| format!("Reading partition layout for {}", disk.path))?;
            // Find the exact device entry
            let devices = tree.blockdevices.into_iter()
                .find(|d| {
                    if let Some(p) = &d.path {
                        p == &disk.path
                    } else {
                        false
                    }
                })
                .with_context(|| format!("device {} not found", disk.path))?;
            // Compare children vs partitions
            if !PartitionManager::compare_partitions(&disk.partitions, &devices.children) {
                return Ok(true)
            }
        }
        // No mismatches
        Ok(false)
    }

    fn apply(&self, cfg: &Config) -> Result<()> {
        for disk in cfg.disks.iter() {
            // Recreate partition table
            CommandRunner::new("parted")
                .arg(&disk.path)
                .arg("mklabel").arg(&disk.table)
                .sudo()
                .run()
                .with_context(|| format!("creating {} label on {}", disk.table, disk.path))?;

            // Create each partition
            for (idx, part) in disk.partitions.iter().enumerate() {
                let mut cmd = CommandRunner::new("parted")
                    .arg(&disk.path)
                    .arg("mkpart")
                    .arg("primary")
                    .arg(part.fs.clone().unwrap_or_else(|| "ext4".into()))
                    .sudo();

                // If a size is given, assume start at 0% for simplicity
                if let Some(size) = part.size_gb {
                    cmd = cmd.arg("0%").arg(&format!("{}MB", (size * 1024.0)));
                }

                cmd.run()
                    .with_context(|| format!("creating partition {} on {}", part.name, disk.path))?;

                // Apply flags (boot, esp, etc.)
                if let Some(flags) = &part.flags {
                    let partition_index = idx + 1; // parted uses 1-based indexes
                    for flag in flags {
                        CommandRunner::new("parted")
                            .arg(&disk.path)
                            .arg("set")
                            .arg(&partition_index.to_string())
                            .arg(flag)
                            .sudo()
                            .run()
                            .with_context(|| {
                                format!(
                                    "setting flag '{}' on partition {} (index {})",
                                    flag, part.name, partition_index
                                )
                            })?;
                    }
                }
            }
        }
        Ok(())
    }
}