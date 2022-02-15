use anyhow::Result;

pub fn install(app_name: &str) -> Result<()> {
    println!("Installing {}", app_name);
    Ok(())
}
