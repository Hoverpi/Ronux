use core::{Config, CommandRunner};
use anyhow::Result;

fn main() -> Result<()> {
    let cfg = Config::load("config/configuration.ron")?;
    
    println!("Load config: {:#?}", cfg);

    let out = CommandRunner::new("lsblk").arg("-f").capture().run()?;
    let changed = cfg.disks.iter().any(|d| !out.contains(&d.path));

    println!("{} \n {}", out, changed);

    Ok(())
}
