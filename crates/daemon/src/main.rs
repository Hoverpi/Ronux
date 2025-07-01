use core::Config;

fn main() -> core::Result<()> {
    let cfg = Config::load("config/configuration.ron")?;
    
    println!("Load config: {:#?}", cfg);
    Ok(())
}
