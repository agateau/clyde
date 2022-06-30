use anyhow::Result;

use crate::app::App;

pub fn update(app: &App) -> Result<()> {
    eprintln!("Updating Clyde store...");
    app.store.update()
}
