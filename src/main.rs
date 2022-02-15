//! Big-Little Matching
//!
//! See the `biglittle` library for more on the matching algorithms used.

use anyhow::bail;
use clap::Parser;
use std::{ffi::OsStr, path::PathBuf};

/// CLI Arguments
#[derive(Clone, Debug, Parser)]
#[clap(about, author, version)]
pub struct Args {
    /// Input Data Path
    ///
    /// If this path is omitted, then the CLI will enter interactive mode.
    pub input: Option<PathBuf>,
}

/// Runs the Big-Little Matching CLI.
#[inline]
pub fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.input {
        Some(path) => match path.extension().and_then(OsStr::to_str) {
            Some("csv") => {
                todo!("Implement CSV parsing.")
            }
            Some(ext) => bail!("Unrecognized input file format: {ext}."),
            _ => bail!("Unable to parse input path: {}.", path.display()),
        },
        _ => {
            todo!("Implement interactive mode.")
        }
    }
}
