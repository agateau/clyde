use anyhow::Result;

use crate::app::App;

pub fn show(app: &App, app_name: &str, list: bool) -> Result<()> {
    let db = &app.database;
    let package = app.store.get_package(app_name)?;

    println!("Name: {}", package.name);
    println!("Description: {}", package.description);
    println!("Homepage: {}", package.homepage);

    if let Some(installed_version) = db.get_package_version(&package.name)? {
        println!("Installed version: {}", installed_version);
    }

    println!();
    println!("Available versions:");
    for (version, builds) in package.releases.iter().rev() {
        let mut arch_os_list = Vec::from_iter(builds.keys().map(|x| format!("{x}")));
        arch_os_list.sort();
        let arch_os_str = arch_os_list.join(", ");
        println!("- {version} ({arch_os_str})");
    }

    if list {
        println!();
        println!("Installed files:");
        let fileset = db.get_package_files(&package.name)?;
        let mut files = Vec::from_iter(fileset);
        files.sort();
        for file in files {
            let path = app.install_dir.join(file);
            println!("- {}", path.display());
        }
    }
    Ok(())
}
