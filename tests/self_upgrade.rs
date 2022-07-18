#[cfg(test)]
mod self_upgrade {
    const CLYDE_YAML_TEMPLATE: &str = "
            name: clyde
            description:
            homepage:
            releases:
              @version@:
                @arch_os@:
                  url: @url@
                  sha256: @sha256@
            installs:
              @version@:
                any:
                  strip: 0
                  files:
                    clyde${exe_ext}: bin/
            ";
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    use anyhow::Result;

    use clyde::app::App;
    use clyde::arch_os::ArchOs;
    use clyde::checksum;

    fn create_clyde_yaml(store_dir: &Path, version: &str) -> Result<()> {
        let clyde_path = env!("CARGO_BIN_EXE_clyde");
        let url = format!("file://{clyde_path}");
        let sha256 = checksum::compute_checksum(Path::new(&clyde_path))?;

        let content = CLYDE_YAML_TEMPLATE
            .replace("@version@", version)
            .replace("@arch_os@", &ArchOs::current().to_str())
            .replace("@url@", &url)
            .replace("@sha256@", &sha256);
        fs::write(store_dir.join("clyde.yaml"), content)?;
        Ok(())
    }

    #[test]
    fn clyde_can_upgrade_itself() {
        // GIVEN a store with Clyde installed
        let clyde_home = assert_fs::TempDir::new().unwrap();

        let store_dir = clyde_home.join("store");
        fs::create_dir(&store_dir).unwrap();
        create_clyde_yaml(&store_dir, "0.1.0").unwrap();

        let app = App::new(&clyde_home).unwrap();
        app.database.create().unwrap();

        let mut cmd = Command::new(env!("CARGO_BIN_EXE_clyde"));
        cmd.env("CLYDE_HOME", clyde_home.path());
        cmd.args(["install", "clyde"]);
        let status = cmd.status().expect("Failed to run clyde");
        assert!(status.success());

        // WHEN a new version of Clyde is available
        create_clyde_yaml(&store_dir, "0.2.0").unwrap();

        // AND the user runs `clyde install clyde`, using the installed clyde executable, meaning
        // the executable has to replace itself
        let mut cmd = Command::new(clyde_home.join("inst").join("bin").join("clyde"));
        cmd.env("CLYDE_HOME", clyde_home.path());
        cmd.args(["install", "clyde"]);

        // THEN clyde updates itself without problems
        let status = cmd.status().expect("Failed to run clyde to upgrade itself");
        assert!(status.success());
    }
}
