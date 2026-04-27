#[cfg(feature = "zip")]
pub use self::zip::Zip;

#[cfg(feature = "zip")]
mod zip {
    use std::fs::{create_dir_all, File};
    use std::io::{copy, Read, Seek};
    use std::path::Path;

    use zip::ZipArchive;

    use crate::{Archive, ArchiverError, Result};

    pub struct Zip<R: Read + Seek> {
        archive: ZipArchive<R>,
    }

    impl Zip<File> {
        pub fn open(path: &Path) -> std::io::Result<Self> {
            let archive = File::open(path)?;

            Self::new(archive)
        }
    }

    impl<R: Read + Seek> Zip<R> {
        pub fn new(r: R) -> std::io::Result<Self> {
            let archive = ZipArchive::new(r)?;

            Ok(Self { archive })
        }
    }

    impl<R: Read + Seek> Archive for Zip<R> {
        fn contains(&mut self, file: String) -> Result<bool> {
            let result = match self.archive.by_name(&file) {
                Ok(_) => true,
                Err(_) => false,
            };

            Ok(result)
        }

        fn extract(&mut self, destination: &Path) -> Result<()> {
            for i in 0..self.archive.len() {
                let mut file = self.archive.by_index(i).unwrap();
                let target = file.mangled_name();
                let target = destination.join(target);

                if (&*file.name()).ends_with('/') {
                    create_dir_all(target)?;
                } else {
                    if let Some(p) = target.parent() {
                        if !p.exists() {
                            create_dir_all(&p)?;
                        }
                    }
                    let mut outfile = File::create(target)?;
                    copy(&mut file, &mut outfile)?;
                }
            }

            Ok(())
        }

        fn extract_single(&mut self, target: &Path, file: String) -> Result<()> {
            if let Some(p) = target.parent() {
                if !p.exists() {
                    create_dir_all(&p)?;
                }
            }

            let mut f = self
                .archive
                .by_name(&file)
                .map_err(|_| ArchiverError::NotFound)?;

            let mut target = File::create(target)?;
            copy(&mut f, &mut target)?;

            return Ok(());
        }

        fn files(&mut self) -> Result<Vec<String>> {
            let files = self.archive.file_names().map(|e| e.into()).collect();

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
