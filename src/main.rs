//! Big-Little Matching
//!
//! See the `biglittle` library for more on the matching algorithms used.

use anyhow::{anyhow, bail, Result};
use biglittle::{Big, Index, Kind, Little, Names, PreferenceTable};
use clap::Parser;
use csv::{Reader, ReaderBuilder, Trim};
use indexmap::IndexMap;
use std::{ffi::OsStr, fs::File, path::PathBuf};

/// CLI Arguments
#[derive(Clone, Debug, Parser)]
#[clap(about, author, version)]
pub struct Args {
    /// Big Input Data Path
    pub big_input: PathBuf,

    /// Little Input Data Path
    pub little_input: PathBuf,
}

///
#[inline]
fn check_input_extension(path: PathBuf) -> Result<PathBuf> {
    match path.extension().and_then(OsStr::to_str) {
        Some("csv") => Ok(path),
        Some(ext) => bail!("Unrecognized input file format: {ext}."),
        _ => bail!("Unable to parse input path: {}.", path.display()),
    }
}

///
#[inline]
fn reader(path: PathBuf) -> Result<Reader<File>> {
    Ok(ReaderBuilder::new()
        .flexible(true)
        .trim(Trim::All)
        .from_path(path)?)
}

///
#[inline]
fn insert_name<K>(names: &mut Names, name: &str) -> Result<Index<K>>
where
    K: Kind,
{
    names.insert::<K>(name.to_string()).ok_or_else(|| {
        anyhow!(
            "Unable to insert {name} into the {:?} name table.",
            K::dynamic()
        )
    })
}

///
#[derive(Default)]
pub struct Records {
    ///
    bigs: IndexMap<String, Vec<String>>,

    ///
    littles: IndexMap<String, Vec<String>>,
}

impl Records {
    ///
    #[inline]
    fn load(big_reader: Reader<File>, little_reader: Reader<File>) -> Result<Self> {
        let mut records = Self::default();
        load_records::<Big>(big_reader, &mut records.bigs)?;
        load_records::<Little>(little_reader, &mut records.littles)?;
        Ok(records)
    }

    ///
    #[inline]
    fn extract_preferences(self, names: &mut Names, table: &mut PreferenceTable) -> Result<()> {
        for big in self.bigs.keys() {
            insert_name::<Big>(names, big)?;
        }
        for little in self.littles.keys() {
            insert_name::<Little>(names, little)?;
        }
        for (_, preferences) in self.bigs {
            table.insert::<Big, _>(
                preferences
                    .iter()
                    .map(|n| insert_name(names, n))
                    .collect::<Result<Vec<_>>>()?,
            );
        }
        for (_, preferences) in self.littles {
            table.insert::<Little, _>(
                preferences
                    .iter()
                    .map(|n| insert_name(names, n))
                    .collect::<Result<Vec<_>>>()?,
            );
        }
        Ok(())
    }
}

///
#[inline]
fn load_records<K>(
    mut reader: Reader<File>,
    records: &mut IndexMap<String, Vec<String>>,
) -> Result<()>
where
    K: Kind,
{
    let start_index = reader
        .headers()?
        .iter()
        .position(|h| h == "Name")
        .ok_or(anyhow!("Missing `Name` header."))?;
    for record in reader.records() {
        let record = record?;
        let mut record = record.iter().skip(start_index);
        let name = record.next().ok_or(anyhow!("Missing `Name` record."))?;
        records.insert(
            name.to_string(),
            record
                .filter(|n| !n.is_empty())
                .map(|s| s.to_string())
                .collect(),
        );
    }
    Ok(())
}

/// Runs the Big-Little Matching CLI.
#[inline]
pub fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let big_reader = reader(check_input_extension(args.big_input)?)?;
    let little_reader = reader(check_input_extension(args.little_input)?)?;
    let mut names = Names::default();
    let mut preferences = PreferenceTable::default();

    let records = Records::load(big_reader, little_reader)?;
    records.extract_preferences(&mut names, &mut preferences)?;

    println!(
        "{}",
        preferences
            .find_even_matching()
            .ok_or(anyhow!("Unable to find fair matching."))?
            .display(&names)
    );

    Ok(())
}
