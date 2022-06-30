use anyhow::Result;

use crate::app::App;

pub fn search(app: &App, query: &str) -> Result<()> {
    let results = app.store.search(query)?;
    if results.is_empty() {
        eprintln!("No packages found matching '{}'", query);
    } else {
        for result in results {
            println!("{}: {}", result.name, result.description);
        }
    }
    Ok(())
}
