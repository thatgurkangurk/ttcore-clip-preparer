use anyhow::{Context, Result};
use dialoguer::Confirm;
use ffmpeg_sidecar::download::{ffmpeg_download_url, unpack_ffmpeg};
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{BufWriter, Read, Write};
use std::path::PathBuf;
use std::time::Duration;
use tempfile::tempdir;

pub struct FfmpegPaths {
    pub ffmpeg: OsString,
    pub ffprobe: OsString,
}

pub fn get_local_ffmpeg_paths() -> FfmpegPaths {
    let bin_dir = env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("data")
        .join("bin");

    let mut ffmpeg_path = bin_dir.join("ffmpeg");
    let mut ffprobe_path = bin_dir.join("ffprobe");

    if cfg!(windows) {
        ffmpeg_path.set_extension("exe");
        ffprobe_path.set_extension("exe");
    }

    FfmpegPaths {
        ffmpeg: ffmpeg_path.into_os_string(),
        ffprobe: ffprobe_path.into_os_string(),
    }
}

pub fn download_ffmpeg_cmd() -> Result<()> {
    let cwd = env::current_dir().context("failed to get current working directory")?;
    let bin_dir = cwd.join("data").join("bin");

    let download_url =
        ffmpeg_download_url().context("failed to determine download url for current platform")?;

    let confirmed = Confirm::new()
        .with_prompt(format!("download ffmpeg binaries from {download_url}?"))
        .default(true)
        .interact()?;

    if !confirmed {
        println!("download cancelled");
        return Ok(());
    }

    let client = reqwest::blocking::Client::new();
    let mut response = client
        .get(download_url)
        .send()
        .context("failed to start download. are you online?")?
        .error_for_status()?;

    let total_size = response.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}",
        )?
        .progress_chars("#>-"),
    );
    pb.set_message("downloading ffmpeg");

    let temp_dir = tempdir()?;
    let archive_path = temp_dir.path().join("ffmpeg_archive");
    let mut dest = BufWriter::new(File::create(&archive_path)?);

    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = response.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        dest.write_all(&buffer[..bytes_read])?;
        pb.inc(bytes_read as u64);
    }
    dest.flush()?;
    pb.finish_with_message("download complete");

    if !bin_dir.exists() {
        fs::create_dir_all(&bin_dir)
            .context("failed to create 'data/bin' directory in working directory")?;
    }

    let unpack_pb = ProgressBar::new_spinner();
    unpack_pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg} [{elapsed_precise}]")?
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    unpack_pb.set_message(format!("unpacking ffmpeg into {}", bin_dir.display()));
    unpack_pb.enable_steady_tick(Duration::from_millis(80));

    let unpack_result = unpack_ffmpeg(&archive_path, &bin_dir);

    match unpack_result {
        Ok(_) => {
            unpack_pb.finish_with_message("unpacking complete");

            write_license_file(&bin_dir, download_url)?;

            println!("ffmpeg successfully installed to {}!", bin_dir.display());
            Ok(())
        }
        Err(e) => {
            unpack_pb.abandon_with_message("unpacking failed");
            Err(e).context("failed to unpack ffmpeg into local data/bin directory")
        }
    }
}

fn write_license_file(bin_dir: &std::path::Path, download_url: &str) -> Result<()> {
    let license_path = bin_dir.join("THIRD_PARTY_LICENSES.txt");

    let binary_filename = download_url
        .rsplit('/')
        .next()
        .unwrap_or("ffmpeg-release-build");

    let target_os = env::consts::OS;
    let target_arch = env::consts::ARCH;

    let license_content = format!(
        r#"==============================================================================
THIRD-PARTY SOFTWARE NOTICES AND INFORMATION
==============================================================================

these are pre-compiled binary executables of ffmpeg and ffprobe, licensed under the LGPL-3.0 licence

------------------------------------------------------------------------------
downloaded from: {download_url}
archive file:    {binary_filename}
target system:   {target_os} ({target_arch})
installed to:    {bin_dir_display}

------------------------------------------------------------------------------
ffmpeg official website: https://ffmpeg.org
ffmpeg source code: https://git.ffmpeg.org/ffmpeg.git
ffmpeg source snapshots: https://ffmpeg.org/download.html#releases
"#,
        download_url = download_url,
        binary_filename = binary_filename,
        target_os = target_os,
        target_arch = target_arch,
        bin_dir_display = bin_dir.display(),
    );

    fs::write(&license_path, license_content)
        .context("failed to write THIRD_PARTY_LICENSES.txt to data/bin")?;

    println!("📄 license details written to {}", license_path.display());

    Ok(())
}
