// Copyright (C) 2022 Red Hat, Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

mod analyzers;
mod html;
mod html_v2;
mod manifest;
mod mustgather;
mod resources;

mod prelude {
    pub use crate::html::*;
    pub use crate::manifest::*;
    pub use crate::mustgather::*;
    pub use anyhow::{Result, anyhow};
}
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use crate::prelude::*;

use clap::Parser;
use flate2::read::GzDecoder;
use tar::Archive;
use tempfile::TempDir;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to the must-gather.
    path: String,
    /// Output file or directory. Site mode defaults to a must-gather-analyze.<timestamp>/
    /// directory beside the must-gather. Single-file mode defaults to output.html.
    output: Option<String>,
    /// Open a must-gather archive in tar format
    #[arg(long)]
    tar: bool,
    /// Write a multi-file static site
    #[arg(long)]
    site: bool,
    /// Write a single self-contained HTML file
    #[arg(long, conflicts_with = "site")]
    single_file: bool,
    /// Use the legacy HTML UI
    #[arg(long, conflicts_with = "single_file")]
    v1: bool,
}

fn strip_known_suffixes(name: &str) -> &str {
    name.strip_suffix(".tar.gz")
        .or_else(|| name.strip_suffix(".tgz"))
        .or_else(|| name.strip_suffix(".tar"))
        .unwrap_or(name)
        .strip_suffix("_unpack")
        .unwrap_or_else(|| {
            name.strip_suffix(".tar.gz")
                .or_else(|| name.strip_suffix(".tgz"))
                .or_else(|| name.strip_suffix(".tar"))
                .unwrap_or(name)
        })
}

fn normalize_must_gather_timestamp(name: &str) -> Option<String> {
    let trimmed = strip_known_suffixes(name);

    if let Some(rest) = trimmed.strip_prefix("must-gather-") {
        let parts: Vec<&str> = rest.split('-').collect();

        if parts.len() >= 6
            && parts[0].len() == 2
            && parts[1].len() == 2
            && parts[2].len() == 4
            && parts[3].len() == 2
            && parts[4].len() == 2
            && parts[5].len() == 2
            && parts[..6]
                .iter()
                .all(|part| part.chars().all(|ch| ch.is_ascii_digit()))
        {
            return Some(format!(
                "{}{}{}-{}{}{}",
                parts[2], parts[0], parts[1], parts[3], parts[4], parts[5]
            ));
        }

        if parts.len() >= 2
            && parts[0].len() == 8
            && parts[1].len() == 6
            && parts[0].chars().all(|ch| ch.is_ascii_digit())
            && parts[1].chars().all(|ch| ch.is_ascii_digit())
        {
            return Some(format!("{}-{}", parts[0], parts[1]));
        }
    }

    None
}

fn default_site_output_dir(path: &str) -> PathBuf {
    let must_gather_path = Path::new(path);
    let basename = must_gather_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path);

    let output_name = match normalize_must_gather_timestamp(basename) {
        Some(timestamp) => format!("must-gather-analyze.{}", timestamp),
        None => "must-gather-analyze.output".to_string(),
    };

    must_gather_path
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(output_name)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.path == "demo" {
        // Special case to render a demo report
        let mg = MustGather::from("testdata/must-gather-valid/sample-openshift-release/")?;

        std::fs::create_dir_all("target/html")?;

        if !cli.v1 {
            let html = html_v2::generate_html(&mg)?;
            std::fs::write("target/html/index.html", html)?;
        } else {
            let index = Html::from(mg)?;
            std::fs::write("target/html/index.html", index.render())?;
        }
    } else {
        let mg;
        let tmp_dir;
        if cli.tar {
            tmp_dir = match extract_tar(&cli.path) {
                Ok(dir) => dir,
                Err(error) => {
                    eprintln!("Could not extract tar file: {}", cli.path);
                    return Err(error);
                }
            };
            mg = MustGather::from(tmp_dir.path().to_str().unwrap())?;
        } else {
            mg = MustGather::from(&cli.path)?;
        }

        if cli.v1 {
            let output_path = cli.output.as_deref().unwrap_or("output.html");
            let index = Html::from(mg)?;
            std::fs::write(output_path, index.render())?;
            eprintln!("Wrote HTML report to {}", output_path);
        } else if cli.single_file {
            let output_path = cli.output.as_deref().unwrap_or("output.html");
            let html = html_v2::generate_html(&mg)?;
            std::fs::write(output_path, &html)?;
            eprintln!("Wrote HTML report to {}", output_path);
        } else {
            let default_output_path;
            let output_path = if let Some(output) = cli.output.as_deref() {
                Path::new(output)
            } else {
                default_output_path = default_site_output_dir(&cli.path);
                default_output_path.as_path()
            };
            html_v2::generate_site(output_path, &mg)?;
            eprintln!("Wrote HTML site to {}/index.html", output_path.display());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_site_output_dir_is_beside_must_gather() {
        assert_eq!(
            default_site_output_dir("/tmp/cases/must-gather-20260526-163424"),
            PathBuf::from("/tmp/cases/must-gather-analyze.20260526-163424")
        );
    }

    #[test]
    fn default_site_output_dir_handles_relative_path() {
        assert_eq!(
            default_site_output_dir("cases/must-gather.local.customer.1234"),
            PathBuf::from("cases/must-gather-analyze.output")
        );
    }
}

// extract_tar extracts tar or tar.gz file into a temporary directory.
// The temporary directory has lifetime of the returned TempDir object.
fn extract_tar(path: &str) -> Result<TempDir> {
    let mut tar_file = File::open(path)?;

    // Infer file type from magic number
    let mut buf = [0; 512];
    tar_file.read_exact(&mut buf)?;
    tar_file.seek(SeekFrom::Start(0))?;
    let kind = match infer::get(&buf) {
        Some(kind) => kind,
        None => Err(anyhow!("Unknown file type"))?,
    };

    // If the file is gzipped, wrap it in a GzDecoder.
    let reader: Box<dyn Read> = match kind.mime_type() {
        "application/gzip" => Box::new(GzDecoder::new(tar_file)),
        "application/x-tar" => Box::new(tar_file),
        _ => Err(anyhow!("Unsupported file type"))?,
    };

    // Unpack the tar file into a temporary directory.
    let mut archive = Archive::new(reader);
    let tmp_dir = TempDir::with_prefix("mga-must-gather")?;
    archive.unpack(tmp_dir.path())?;
    Ok(tmp_dir)
}
