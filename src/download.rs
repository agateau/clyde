// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, Result};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::ClientBuilder;
use reqwest::Url;

const FILE_PREFIX: &str = "file://";

const PROGRESS_BAR_TEMPLATE: &str =
    "{bar:40.dim.green/blue} {bytes}/{total_bytes} - {bytes_per_sec}";

struct ProgressWriter<W: Write> {
    writer: W,
    downloaded: u64,
    bar: ProgressBar,
}

impl<W> ProgressWriter<W>
where
    W: Write,
{
    fn new(writer: W, total_size: u64) -> Self {
        let bar = ProgressBar::new(total_size);
        bar.set_style(
            ProgressStyle::default_bar()
                .template(PROGRESS_BAR_TEMPLATE)
                .progress_chars("━━─"),
        );
        Self {
            writer,
            downloaded: 0,
            bar,
        }
    }

    fn print_progress(&mut self) {
        self.bar.set_position(self.downloaded);
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

fn create_partial_path_name(path: &Path) -> Result<PathBuf> {
    let name = path
        .file_name()
        .ok_or_else(|| anyhow!("{path:?} has no filename"))?;
    let mut name = name.to_os_string();
    name.push(".partial");
    Ok(path.with_file_name(name))
}

fn https_download(url_str: &str, dst_path: &Path) -> Result<()> {
    eprintln!("Downloading {url_str} to {dst_path:?}");
    let partial_path = create_partial_path_name(dst_path)?;
    let mut file = File::create(&partial_path)?;

    let url = Url::parse(url_str)?;

    let client_builder = ClientBuilder::new().timeout(Duration::from_secs(3));
    let client = client_builder.build()?;

    let mut response = client.get(url).send()?.error_for_status()?;
    if let Some(total_size) = response.content_length() {
        let mut writer = ProgressWriter::new(&mut file, total_size);
        response.copy_to(&mut writer)?;
    } else {
        response.copy_to(&mut file)?;
    }
    fs::rename(partial_path, dst_path)?;

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
