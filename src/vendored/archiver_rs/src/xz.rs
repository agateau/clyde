//#[cfg(feature = "xz")]
pub use self::xz::Xz;

//#[cfg(feature = "xz")]
#[allow(clippy::module_inception)]
mod xz {
    use std::fs::{create_dir_all, File};
    use std::io::{copy, Read};
    use std::path::Path;

    use xz2::read::XzDecoder;

    use crate::vendored::archiver_rs::{Compressed, Result};

    pub struct Xz<R: Read> {
        archive: XzDecoder<R>,
    }

    impl Xz<File> {
        pub fn open(path: &Path) -> std::io::Result<Self> {
            let archive = File::open(path)?;

            Self::new(archive)
        }
    }

    impl<R: Read> Xz<R> {
        pub fn new(r: R) -> std::io::Result<Self> {
            let archive = XzDecoder::new(r);

            Ok(Self { archive })
        }
    }

    impl<R: Read> Compressed for Xz<R> {
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

    impl<R: Read> Read for Xz<R> {
        fn read(&mut self, into: &mut [u8]) -> std::io::Result<usize> {
            self.archive.read(into)
        }
    }
}
