#[cfg(feature = "tar")]
pub use self::tar::Tar;

#[cfg(feature = "tar")]
mod tar {
    use std::fs::{create_dir_all, File};
    use std::io::{copy, Read};
    use std::path::Path;

    use crate::{Archive, ArchiverError, Result};

    pub struct Tar<R: Read> {
        archive: tar::Archive<R>,
    }

    impl Tar<File> {
        pub fn open(path: &Path) -> std::io::Result<Self> {
            let archive = File::open(path)?;

            Self::new(archive)
        }
    }

    impl<R: Read> Tar<R> {
        pub fn new(r: R) -> std::io::Result<Self> {
            let archive = tar::Archive::new(r);

            Ok(Self { archive })
        }
    }

    impl<R: Read> Archive for Tar<R> {
        fn contains(&mut self, file: String) -> Result<bool> {
            for f in self.archive.entries()? {
                let f = f?;
                let name = f.path()?;
                let name = name.to_str().unwrap_or_else(|| "");

                if name == file {
                    return Ok(true);
                }
            }

            Ok(false)
        }

        fn extract(&mut self, destination: &Path) -> Result<()> {
            if !destination.exists() {
                create_dir_all(destination)?;
            }

            self.archive.unpack(destination)?;

            Ok(())
        }

        fn extract_single(&mut self, target: &Path, file: String) -> Result<()> {
            if let Some(p) = target.parent() {
                if !p.exists() {
                    create_dir_all(&p)?;
                }
            }

            for f in self.archive.entries()? {
                let mut f = f?;
                let name = f.path()?;

                if name.to_str().unwrap_or_else(|| "") == &file {
                    let mut target = File::create(target)?;
                    copy(&mut f, &mut target)?;

                    return Ok(());
                }
            }

            Err(ArchiverError::NotFound)?
        }

        fn files(&mut self) -> Result<Vec<String>> {
            let files = self
                .archive
                .entries()?
                .into_iter()
                .map(|e| e.unwrap().path().unwrap().to_str().unwrap().into())
                .collect();

            Ok(files)
        }

        fn walk(&mut self, f: Box<dyn Fn(String) -> Option<String>>) -> Result<()> {
            let files = self.files()?;

            for file in files {
                if let Some(f) = f(file.clone()) {
                    self.extract_single(Path::new(&f), file.clone())?;
                }
            }

            Ok(())
        }
    }
}
