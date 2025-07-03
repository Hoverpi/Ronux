// declare modules
pub mod model;           // <– parser
pub mod traits;           // <– Configurable
pub mod orchestrator;     // <– run_pipeline()
pub mod stages;          // <– all the installers
pub mod utils;          // <- Centralize the commands

// re‑exports for easy use
pub use model::{Config, Lsblk};

pub use model::{LsblkOutput, BlockDevice, Firmware, Disk, Partition, Lvm, LogicalVolume, Bootloader, Kernel};

// pub use orchestrator::run_pipeline;
pub use traits::Configurable;

pub use utils::CommandRunner;