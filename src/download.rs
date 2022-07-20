// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cmp::Ordering;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;

use anyhow::{anyhow, Result};
use reqwest::blocking::ClientBuilder;
use reqwest::Url;

const PROGRESS_LENGTH: usize = 40;

const FILE_PREFIX: &str = "file://";

struct ProgressWriter<W: Write> {
    writer: W,
    total_size: u64,
    downloaded: u64,
    progress_ratio: usize,
}

impl<W> ProgressWriter<W>
where
    W: Write,
{
    fn new(writer: W, total_size: u64) -> Self {
        Self {
            writer,
            total_size,
            downloaded: 0,
            progress_ratio: 0,
        }
    }

    fn print_progress(&mut self) {
        let ratio = (PROGRESS_LENGTH * self.downloaded as usize) / self.total_size as usize;
        if ratio == self.progress_ratio {
            return;
        }
        self.progress_ratio = ratio;

        if self.downloaded < self.total_size {
            let mut progress_str = String::with_capacity(PROGRESS_LENGTH);
            for idx in 0..PROGRESS_LENGTH {
                let ch = match idx.cmp(&ratio) {
                    Ordering::Less => '=',
                    Ordering::Equal => '>',
                    Ordering::Greater => ' ',
                };
                progress_str.push(ch);
            }
            eprint!("  [{}]\r", progress_str);
        } else {
            eprint!("   {} \r", " ".repeat(PROGRESS_LENGTH));
        }
    }
}

impl<W: Write> Write for ProgressWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice]) -> io::Result<usize> {
        self.writer.write_vectored(bufs)
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.writer.write_all(buf).map(|()| {
            self.downloaded += buf.len() as u64;
            self.print_progress();
        })
    }
}

pub fn download(url_str: &str, dst_path: &Path) -> Result<()> {
    if url_str.starts_with("http://") || url_str.starts_with("https://") {
        https_download(url_str, dst_path)
    } else if url_str.starts_with(FILE_PREFIX) {
        file_download(url_str, dst_path)
    } else {
        Err(anyhow!("Unspported URL protocol: {url_str}"))
    }
}

fn https_download(url_str: &str, dst_path: &Path) -> Result<()> {
    eprintln!("Downloading {url_str} to {dst_path:?}");
    let mut file = File::create(dst_path)?;

    let url = Url::parse(url_str)?;

    let client_builder = ClientBuilder::new().timeout(Duration::from_secs(3));
    let client = client_builder.build()?;

    let mut response = client.get(url).send()?.error_for_status()?;
    if let Some(total_size) = response.content_length() {
        eprintln!("  {} bytes", total_size);
        let mut writer = ProgressWriter::new(&mut file, total_size);
        response.copy_to(&mut writer)?;
    } else {
        eprintln!("  Unknown size");
        response.copy_to(&mut file)?;
    }

    Ok(())
}

// This one is mainly useful for tests
fn file_download(url_str: &str, dst_path: &Path) -> Result<()> {
    eprintln!("Copying {url_str} to {dst_path:?}");
    let path_str = url_str
        .strip_prefix(FILE_PREFIX)
        .unwrap_or_else(|| panic!("File URL ({url_str}) does not start with {FILE_PREFIX}"));
    let mut file = File::open(path_str)?;
    let total_size = fs::metadata(path_str)?.len();

    let mut dst_file = File::create(dst_path)?;
    let mut writer = ProgressWriter::new(&mut dst_file, total_size);

    io::copy(&mut file, &mut writer)?;
    Ok(())
}
