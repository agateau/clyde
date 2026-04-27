// #[cfg(feature = "bzip")]
pub use self::bzip2::Bzip2;

// #[cfg(feature = "bzip")]
#[allow(clippy::module_inception)]
mod bzip2 {
    use std::fs::{create_dir_all, File};
    use std::io::{copy, Read};
    use std::path::Path;

    use bzip2::read::BzDecoder;

    use crate::vendored::archiver_rs::{Compressed, Result};

    pub struct Bzip2<R: Read> {
        archive: BzDecoder<R>,
    }

    impl Bzip2<File> {
        pub fn open(path: &Path) -> std::io::Result<Self> {
            let archive = File::open(path)?;

            Self::new(archive)
        }
    }

    impl<R: Read> Bzip2<R> {
        pub fn new(r: R) -> std::io::Result<Self> {
            let archive = BzDecoder::new(r);

            Ok(Self { archive })
        }
    }

    impl<R: Read> Compressed for Bzip2<R> {
        fn decompress(&mut self, target: &Path) -> Result<()> {
            if let Some(p) = target.parent() {
                if !p.exists() {
                    create_dir_all(p)?;
                }
            }

            let mut output = File::create(target)?;
            copy(&mut self.archive, &mut output)?;

            Ok(())
        }
    }

    impl<R: Read> Read for Bzip2<R> {
        fn read(&mut self, into: &mut [u8]) -> std::io::Result<usize> {
            self.archive.read(into)
        }
    }
}
