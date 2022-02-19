//! Big-Little Matching
//!
//! See the `biglittle` library for more on the matching algorithm.

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

/// Checks that the input has the correct extension for CSV parsing.
#[inline]
fn check_input_extension(path: PathBuf) -> Result<PathBuf> {
    match path.extension().and_then(OsStr::to_str) {
        Some("csv") => Ok(path),
        Some(ext) => bail!("Unrecognized input file format: {ext}."),
        _ => bail!("Unable to parse input path: {}.", path.display()),
    }
}

/// Returns a CSV reader for `path`.
#[inline]
fn reader(path: PathBuf) -> Result<Reader<File>> {
    Ok(ReaderBuilder::new()
        .flexible(true)
        .trim(Trim::All)
        .from_path(check_input_extension(path)?)?)
}

/// Inserts `name` into the `names` table returning its index.
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

/// Gets the index of `name` from the `names` table.
#[inline]
fn get_index<K>(names: &Names, name: &str) -> Result<Index<K>>
where
    K: Kind,
{
    names.index::<K>(name).ok_or_else(|| {
        anyhow!(
            "Unable to get {name} from the {:?} name table.",
            K::dynamic()
        )
    })
}

/// Loads the records from `reader` into `records` with the known type `K`.
#[inline]
fn load_from_reader<K>(mut reader: Reader<File>) -> Result<IndexMap<String, Vec<String>>>
where
    K: Kind,
{
    let start_index = reader
        .headers()?
        .iter()
        .position(|h| h == "Name")
        .ok_or(anyhow!("Missing `Name` header."))?;
    let mut records = IndexMap::default();
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
    Ok(records)
}

/// Loads the names and preferences from the `bigs` and `littles` readers.
#[inline]
fn load(bigs: Reader<File>, littles: Reader<File>) -> Result<(Names, PreferenceTable)> {
    let bigs = load_from_reader::<Big>(bigs)?;
    let littles = load_from_reader::<Little>(littles)?;
    let mut names = Names::default();
    let mut table = PreferenceTable::default();
    for big in bigs.keys() {
        insert_name::<Big>(&mut names, big)?;
    }
    for little in littles.keys() {
        insert_name::<Little>(&mut names, little)?;
    }
    for (_, preferences) in bigs {
        table.insert::<Big, _>(
            preferences
                .iter()
                .map(|n| get_index(&names, n))
                .collect::<Result<Vec<_>>>()?,
        );
    }
    for (_, preferences) in littles {
        table.insert::<Little, _>(
            preferences
                .iter()
                .map(|n| get_index(&names, n))
                .collect::<Result<Vec<_>>>()?,
        );
    }
    Ok((names, table))
}

/// Runs the Big-Little Matching CLI.
#[inline]
pub fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let (names, preferences) = load(reader(args.big_input)?, reader(args.little_input)?)?;
    println!(
        "{}",
        preferences
            .find_even_matching()
            .ok_or(anyhow!("Unable to find fair matching."))?
            .display(&names)
    );
    Ok(())
}
