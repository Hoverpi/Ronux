use core::{CommandRunner, Config, Configurable, Lsblk};
use anyhow::{Result, Context};

pub struct LvmManager;

impl Configurable for LvmManager {
    fn verify(&self, cfg: &Config) -> Result<bool> {
        // If no LVM section, nothing to do
        let lvm_cfg = match &cfg.lvm {
            Some(l) => l,
            None => return Ok(false),
        };
        // Check vg exists
        let vgs_out = CommandRunner::new("vgs")
            .arg("--noheadings")
            .arg("-o vg_name")
            .capture()
            .run()
            .context("listing volume groups")?;

        if !vgs_out.lines().any(|vg| vg.trim() == lvm_cfg.vg) {
            // VG missing => need apply
            return Ok(true);
        }
        // Check each LV exists in that VG
        let lvs_out = CommandRunner::new("lvs")
            .arg("--noheadings")
            .arg("-o lv_name,vg_name")
            .capture()
            .run()
            .context("listing logical volumes")?;
        // Build a set of existing LV names for our VG
        let existing_lvs: std::collections::HashSet<_> = lvs_out
            .lines()
            .filter_map(|line| {
                let parts: Vec<_> = line.split_whitespace().collect();
                if parts.get(1).map(|&vg| vg == lvm_cfg.vg).unwrap_or(false) {
                    parts.get(0).map(|&lv| lv.to_string())
                } else {
                    None
                }
            })
            .collect();
        // If any desired LV missing => need apply
        for vol in &lvm_cfg.volumes {
            if !existing_lvs.contains(&vol.name) {
                return Ok(true);
            }
        }

        // All present => nothing to do
        Ok(false)
    }

    fn apply(&self, cfg: &Config) -> Result<()> {
        let lvm_cfg = match &cfg.lvm {
            Some(l) => l,
            None => return Ok(()),
        };

        // Collect PV paths: each partition with luks=true
        let mut pvs = Vec::new();
        for disk in &cfg.disks {
            // Load the block‐device tree for this disk
            let tree = Lsblk::load(&disk.path)
                .with_context(|| format!("reading LVM partitions on {}", disk.path))?;

            // Find the disk entry by full path
            let parent = tree.blockdevices.into_iter()
                .find(|d| d.path.as_deref() == Some(&disk.path))
                .with_context(|| format!("disk {} not found in lsblk output", disk.path))?;

            // For each partition marked luks, grab its mapped path
            for part in &disk.partitions {
                if part.luks.unwrap_or(false) {
                    // find the child device (e.g. cryptlvm)
                    let child_path = parent.children.iter()
                        .find(|c| c.name.ends_with(&part.name))
                        .and_then(|c| c.path.clone())
                        .with_context(|| format!("LUKS partition '{}' not found under {}", part.name, disk.path))?;
                    pvs.push(child_path);
                }
            }
        }

        // Initialize each PV
        for pv in &pvs {
            CommandRunner::new("pvcreate")
                .arg("-y")
                .arg(pv)
                .sudo()
                .run()
                .with_context(|| format!("pvcreate {}", pv))?;
        }

        // Create the VG if it doesn't exist (vgcreate will error if already present)
        CommandRunner::new("vgcreate")
            .arg(&lvm_cfg.vg)
            .args(&pvs)
            .sudo()
            .run()
            .with_context(|| format!("vgcreate {}", lvm_cfg.vg))?;

        // Create each LV with the desired size
        for vol in &lvm_cfg.volumes {
            // Choose size: either specified in GB or use 100%FREE
            let size_arg = vol.size_gb
                .map(|s| format!("{}g", s))
                .unwrap_or_else(|| "100%FREE".into());

            CommandRunner::new("lvcreate")
                .arg("-n").arg(&vol.name)
                .arg("-L").arg(&size_arg)
                .arg(&lvm_cfg.vg)
                .sudo()
                .run()
                .with_context(|| format!("lvcreate {} with size {} in {}", vol.name, size_arg, lvm_cfg.vg))?;
        }

        Ok(())
    }
}