# archiver-rs

[![Crates.io](https://img.shields.io/crates/v/archiver-rs)](http://crates.io/crates/archiver-rs)
[![Docs.rs](https://docs.rs/archiver-rs/badge.svg)](https://docs.rs/archiver-rs)
[![Crates.io](https://img.shields.io/crates/d/archiver-rs)](http://crates.io/crates/archiver-rs)
[![Crates.io](https://img.shields.io/crates/l/archiver-rs)](https://github.com/JoyMoe/archiver-rs/blob/master/LICENSE)

A library for easy interaction with multiple archive formats

## Usage

```rust
let mut bz = archiver_rs::Bzip2::open("foo.tar.bz2");
bz.decompress("foo.tar");

let mut tar = archiver_rs::Tar::open("foo.tar");
tar.files();
tar.contains("bar.txt");
tar.extract("./foo/");
tar.extract_single("./foo/bar.txt", "bar.txt");
```

## License

The MIT License

More info see [LICENSE](LICENSE)
