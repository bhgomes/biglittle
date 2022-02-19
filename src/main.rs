//! Big-Little Matching
//!
//! See the `biglittle` library for more on the matching algorithms used.

use anyhow::bail;
use biglittle::{Big, DynamicKind, Little, Names, PreferenceTable};
use clap::Parser;
use serde::Deserialize;
use std::{ffi::OsStr, fs::File, path::PathBuf};

/// CLI Arguments
#[derive(Clone, Debug, Parser)]
#[clap(about, author, version)]
pub struct Args {
    /// Input Data Path
    pub input: PathBuf,
}

/// Table Record
#[derive(Debug, Deserialize)]
pub struct Record {
    /// Name
    #[serde(rename = "Name")]
    pub name: String,

    /// Matching Kind
    #[serde(rename = "Kind")]
    pub kind: DynamicKind,

    /// Preferences
    #[serde(rename = "Preferences")]
    pub preferences: PreferenceString,
}

///
#[derive(Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct PreferenceString(Vec<String>);

impl TryFrom<String> for PreferenceString {
    type Error = usize;

    #[inline]
    fn try_from(string: String) -> Result<Self, Self::Error> {
        todo!()
    }
}

/// Runs the Big-Little Matching CLI.
#[inline]
pub fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let path = args.input;
    match path.extension().and_then(OsStr::to_str) {
        Some("csv") => {
            let mut names = Names::default();
            let mut preferences = PreferenceTable::default();
            let mut reader = csv::Reader::from_reader(File::open(path)?);
            for result in reader.deserialize::<Record>() {
                let record = result?;
                println!("Inserting Record: {:?}", record);
                names.insert(record.name);
                // FIXME: Check kinds when inserting.
                match record.kind {
                    DynamicKind::Big => {
                        preferences.insert::<Big, _>(
                            record
                                .preferences
                                .0
                                .into_iter()
                                .map(|name| names.insert(name).into()),
                        );
                    }
                    DynamicKind::Little => {
                        preferences.insert::<Little, _>(
                            record
                                .preferences
                                .0
                                .into_iter()
                                .map(|name| names.insert(name).into()),
                        );
                    }
                }
            }
            println!("Names: {:#?}", names);
            println!("Preferences {:#?}", preferences);
            println!("Matching: {:#?}", preferences.find_primitive_matching());
            Ok(())
        }
        Some(ext) => bail!("Unrecognized input file format: {ext}."),
        _ => bail!("Unable to parse input path: {}.", path.display()),
    }
}
