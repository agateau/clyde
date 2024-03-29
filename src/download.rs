// SPDX-FileCopyrightText: 2022 Aurélien Gâteau <mail@agateau.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::{self, File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use reqwest::{header, Error as ReqwestError, StatusCode, Url};

use crate::file_utils;
use crate::ui::Ui;

const FILE_PREFIX: &str = "file://";

const PROGRESS_BAR_TEMPLATE: &str = "[{bar:40}] {bytes} / {total_bytes} - {bytes_per_sec}";

const DOWNLOAD_ATTEMPTS: u64 = 3;

struct ProgressWriter<W: Write> {
    writer: W,
    start_size: u64,
    downloaded: u64,
    bar: ProgressBar,
}

impl<W> ProgressWriter<W>
where
    W: Write,
{
    fn new(ui: &Ui, writer: W, start_size: u64, total_size: u64) -> Self {
        let bar = ProgressBar::new(start_size + total_size);
        let template = ui.get_indent() + PROGRESS_BAR_TEMPLATE;
        bar.set_style(
            ProgressStyle::default_bar()
                .template(&template)
                .unwrap()
                .progress_chars("●●."),
        );
        Self {
            writer,
            start_size,
            downloaded: 0,
            bar,
        }
    }

    fn print_progress(&mut self) {
        self.bar.set_position(self.start_size + self.downloaded);
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

pub fn download(ui: &Ui, url_str: &str, dst_path: &Path) -> Result<()> {
    if url_str.starts_with("http://") || url_str.starts_with("https://") {
        https_download(ui, url_str, dst_path)
    } else if url_str.starts_with(FILE_PREFIX) {
        file_download(ui, url_str, dst_path)
    } else {
        Err(anyhow!("Unspported URL protocol: {url_str}"))
    }
}

fn https_download_internal(ui: &Ui, client: &Client, url: &Url, partial_path: &Path) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(partial_path)
        .with_context(|| format!("Could not create {partial_path:?}"))?;
    file.seek(SeekFrom::End(0))?;
    let mut partial_size = file.stream_position().unwrap_or(0);

    let request = client
        .get(url.clone())
        .header(header::RANGE, format!("bytes={partial_size}-"))
        .send()?;
    let mut response = request.error_for_status()?;

    // Download
    if partial_size > 0 && response.status() == StatusCode::OK {
        // Server does not support ranges (otherwise status() would be
        // StatusCode::PARTIAL_CONTENT). Reset partial file.
        ui.info("Server does not support ranges. Restarting download.");
        file.rewind()?;
        partial_size = 0;
    }
    if let Some(total_size) = response.content_length() {
        let mut writer = ProgressWriter::new(&ui.nest(), file, partial_size, total_size);
        response.copy_to(&mut writer)?;
    } else {
        response.copy_to(&mut file)?;
    }
    Ok(())
}

fn https_download(ui: &Ui, url_str: &str, dst_path: &Path) -> Result<()> {
    let name = file_utils::get_file_name(dst_path)?;
    let partial_path = dst_path.with_file_name(name.to_string() + ".partial");

    let url = Url::parse(url_str)?;
    let client = Client::new();

    for attempt in 1..(DOWNLOAD_ATTEMPTS + 1) {
        if attempt > 1 {
            ui.warn(&format!(
                "Timeout while downloading, retrying (attempt {attempt} / {DOWNLOAD_ATTEMPTS})"
            ));
        }
        ui.info(&format!("Downloading {name}"));
        match https_download_internal(ui, &client, &url, &partial_path) {
            Ok(()) => break,
            Err(err) => match err.downcast_ref::<ReqwestError>() {
                Some(req_err) => {
                    if !req_err.is_timeout() {
                        return Err(err);
                    }
                }
                None => {
                    return Err(err);
                }
            },
        };
    }

    // Done
    fs::rename(partial_path, dst_path)?;

    Ok(())
}

// This one is mainly useful for tests
fn file_download(ui: &Ui, url_str: &str, dst_path: &Path) -> Result<()> {
    ui.info(&format!("Copying {url_str} to {dst_path:?}"));
    let path_str = url_str
        .strip_prefix(FILE_PREFIX)
        .unwrap_or_else(|| panic!("File URL ({url_str}) does not start with {FILE_PREFIX}"));
    let mut file = File::open(path_str)?;
    let total_size = fs::metadata(path_str)?.len();

    let mut dst_file = File::create(dst_path)?;
    let mut writer = ProgressWriter::new(&ui.nest(), &mut dst_file, 0, total_size);

    io::copy(&mut file, &mut writer)?;
    Ok(())
}
