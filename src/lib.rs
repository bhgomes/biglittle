//! Big-Little Matching

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![forbid(rustdoc::broken_intra_doc_links)]
#![forbid(missing_docs)]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::{cmp::Ordering, fmt, marker::PhantomData, num::NonZeroU32};
use indexmap::{map::Entry, IndexMap, IndexSet};

/// Sealed Module
mod sealed {
    /// Sealed Trait
    pub trait Sealed {}
}

/// Dynamic Kind
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DynamicKind {
    /// Big Kind
    Big,

    /// Little Kind
    Little,
}

/// Matching Kind
pub trait Kind: sealed::Sealed + Sized {
    /// Opposite Kind
    type Opposite: Kind;

    /// Returns the [`DynamicKind`] that matches `Self`.
    fn dynamic() -> DynamicKind;

    ///
    fn names(names: &Names) -> NamesSubset<Self>;

    ///
    fn names_mut(names: &mut Names) -> NamesSubsetMut<Self>;

    /// Returns a shared references to the subset of the preference `table` which corresponds to
    /// `Self` preferences.
    fn preferences(table: &PreferenceTable) -> PreferenceSubset<Self>;

    /// Returns a mutable references to the subset of the preference `table` which corresponds to
    /// `Self` preferences.
    fn preferences_mut(table: &mut PreferenceTable) -> PreferenceSubsetMut<Self>;
}

/// Matching Index
#[derive(derivative::Derivative)]
#[derivative(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Index<K>
where
    K: Kind,
{
    /// Index
    index: u32,

    /// Type Parameter Marker
    __: PhantomData<K>,
}

impl<K> Index<K>
where
    K: Kind,
{
    /// Builds a new [`Index`] from `index`.
    #[inline]
    pub fn new(index: u32) -> Self {
        Self {
            index,
            __: PhantomData,
        }
    }

    /// Returns the preference rank of `other` for `self` using `table`.
    #[inline]
    pub fn preference(
        self,
        other: Index<K::Opposite>,
        table: &PreferenceTable,
    ) -> Option<Preference<K>> {
        K::preferences(table).0[self.index as usize]
            .iter()
            .position(|i| *i == other)
            .and_then(|i| NonZeroU32::new((i + 1) as u32).map(Preference::new))
    }

    /// Finds the maximum prefered index among `others` according to `self` using `table`.
    #[inline]
    pub fn max_preference<I>(
        self,
        others: I,
        table: &PreferenceTable,
    ) -> Option<(Index<K::Opposite>, Preference<K>)>
    where
        I: IntoIterator<Item = Index<K::Opposite>>,
    {
        // TODO: Use `Iterator::reduce` to simplify this.
        let mut maximum = None;
        for index in others {
            if let Some(preference) = self.preference(index, table) {
                match maximum.as_mut() {
                    Some((max_index, max_preference)) => {
                        if preference > *max_preference {
                            *max_index = index;
                            *max_preference = preference;
                        }
                    }
                    _ => maximum = Some((index, preference)),
                }
            }
        }
        maximum
    }
}

impl<K> fmt::Debug for Index<K>
where
    K: Kind,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.index.fmt(f)
    }
}

impl<K> From<u32> for Index<K>
where
    K: Kind,
{
    #[inline]
    fn from(index: u32) -> Self {
        Self::new(index)
    }
}

impl<K> From<usize> for Index<K>
where
    K: Kind,
{
    #[inline]
    fn from(index: usize) -> Self {
        Self::new(index as u32)
    }
}

/// Matching Preference
#[derive(derivative::Derivative)]
#[derivative(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Preference<K>
where
    K: Kind,
{
    /// Preference
    preference: NonZeroU32,

    /// Type Parameter Marker
    __: PhantomData<K>,
}

impl<K> Preference<K>
where
    K: Kind,
{
    /// Builds a new [`Preference`] from `preference`.
    #[inline]
    pub fn new(preference: NonZeroU32) -> Self {
        Self {
            preference,
            __: PhantomData,
        }
    }
}

impl<K> fmt::Debug for Preference<K>
where
    K: Kind,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.preference.fmt(f)
    }
}

/// Builds an implementation of [`Kind`] markers.
macro_rules! impl_kind {
    ($doc:expr, $type:ident, $opposite:ident, $index:ident, $preference:ident) => {
        #[doc = $doc]
        #[doc = "Matching Kind Marker"]
        #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $type;

        impl sealed::Sealed for $type {}

        impl Kind for $type {
            type Opposite = $opposite;

            #[inline]
            fn dynamic() -> DynamicKind {
                DynamicKind::$type
            }

            #[inline]
            fn names(names: &Names) -> NamesSubset<Self> {
                NamesSubset::<Self>::new(names)
            }

            #[inline]
            fn names_mut(names: &mut Names) -> NamesSubsetMut<Self> {
                NamesSubsetMut::<Self>::new(names)
            }

            #[inline]
            fn preferences(table: &PreferenceTable) -> PreferenceSubset<Self> {
                PreferenceSubset::<Self>::new(table)
            }

            #[inline]
            fn preferences_mut(table: &mut PreferenceTable) -> PreferenceSubsetMut<Self> {
                PreferenceSubsetMut::<Self>::new(table)
            }
        }

        #[doc = $doc]
        #[doc = "Matching Index Type"]
        pub type $index = Index<$type>;

        #[doc = $doc]
        #[doc = "Matching Preference Type"]
        pub type $preference = Preference<$type>;
    };
}

impl_kind!("Big", Big, Little, BigIndex, BigPreference);
impl_kind!("Little", Little, Big, LittleIndex, LittlePreference);

///
type NamesSubsetType = IndexSet<String>;

///
pub struct NamesSubset<'k, K>(&'k NamesSubsetType, PhantomData<K>)
where
    K: Kind;

impl<'k> NamesSubset<'k, Big> {
    ///
    #[inline]
    fn new(names: &'k Names) -> Self {
        Self(&names.bigs, PhantomData)
    }
}

impl<'k> NamesSubset<'k, Little> {
    ///
    #[inline]
    fn new(names: &'k Names) -> Self {
        Self(&names.littles, PhantomData)
    }
}

///
pub struct NamesSubsetMut<'k, K>(&'k mut NamesSubsetType, PhantomData<K>)
where
    K: Kind;

impl<'k> NamesSubsetMut<'k, Big> {
    ///
    #[inline]
    fn new(names: &'k mut Names) -> Self {
        Self(&mut names.bigs, PhantomData)
    }
}

impl<'k> NamesSubsetMut<'k, Little> {
    ///
    #[inline]
    fn new(names: &'k mut Names) -> Self {
        Self(&mut names.littles, PhantomData)
    }
}

/// Names
#[derive(Debug, Default)]
pub struct Names {
    /// Big Names
    bigs: IndexSet<String>,

    /// Little Names
    littles: IndexSet<String>,
}

impl Names {
    ///
    #[inline]
    pub fn insert<K>(&mut self, name: String) -> Option<Index<K>>
    where
        K: Kind,
    {
        if K::Opposite::names(self).0.contains(&name) {
            return None;
        }
        let names = K::names_mut(self).0;
        names.insert(name.clone());
        self.index(&name)
    }

    ///
    #[inline]
    pub fn get<K>(&self, index: Index<K>) -> Option<&String>
    where
        K: Kind,
    {
        K::names(self).0.get_index(index.index as usize)
    }

    ///
    #[inline]
    pub fn index<K>(&self, name: &str) -> Option<Index<K>>
    where
        K: Kind,
    {
        K::names(self).0.get_index_of(name).map(Into::into)
    }
}

/// Preference Table Subset Type
type PreferenceSubsetType<K> = IndexSet<Vec<Index<<K as Kind>::Opposite>>>;

/// Preference Table Subset
pub struct PreferenceSubset<'k, K>(&'k PreferenceSubsetType<K>)
where
    K: Kind;

impl<'k> PreferenceSubset<'k, Big> {
    /// Builds a new [`Big`] `table` subset.
    #[inline]
    fn new(table: &'k PreferenceTable) -> Self {
        Self(&table.big_preferences)
    }
}

impl<'k> PreferenceSubset<'k, Little> {
    /// Builds a new [`Little`] `table` subset.
    #[inline]
    fn new(table: &'k PreferenceTable) -> Self {
        Self(&table.little_preferences)
    }
}

/// Preference Table Subset
pub struct PreferenceSubsetMut<'k, K>(&'k mut PreferenceSubsetType<K>)
where
    K: Kind;

impl<'k> PreferenceSubsetMut<'k, Big> {
    /// Builds a new [`Big`] `table` subset.
    #[inline]
    fn new(table: &'k mut PreferenceTable) -> Self {
        Self(&mut table.big_preferences)
    }
}

impl<'k> PreferenceSubsetMut<'k, Little> {
    /// Builds a new [`Little`] `table` subset.
    #[inline]
    fn new(table: &'k mut PreferenceTable) -> Self {
        Self(&mut table.little_preferences)
    }
}

/// Matching Preference Table
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PreferenceTable {
    /// Big Preferences
    big_preferences: PreferenceSubsetType<Big>,

    /// Little Preferences
    little_preferences: PreferenceSubsetType<Little>,
}

impl PreferenceTable {
    ///
    #[inline]
    pub fn insert<K, I>(&mut self, preferences: I)
    where
        K: Kind,
        I: IntoIterator<Item = Index<K::Opposite>>,
    {
        K::preferences_mut(self)
            .0
            .insert(Vec::from_iter(preferences));
    }

    ///
    #[inline]
    fn update_matching<'i, I>(&self, matching_set: &mut MatchingSet, little: LittleIndex, bigs: I)
    where
        I: IntoIterator<Item = &'i BigIndex>,
    {
        let mut bigs = bigs.into_iter();
        loop {
            if let Some(big) = bigs.next() {
                if big.preference(little, self).is_some() {
                    matching_set.insert_match(self, *big, little);
                    break;
                }
            } else {
                matching_set.unmatched_littles.insert(little);
                break;
            }
        }
    }

    ///
    #[inline]
    fn collect_unmatched_bigs(&self, matching_set: &mut MatchingSet) {
        for big in 0..self.big_preferences.len() {
            let big = Index::from(big);
            if !matching_set.matches.iter().any(|m| m.big == big) {
                matching_set.unmatched_bigs.insert(big);
            }
        }
    }

    ///
    #[inline]
    fn primitive_matching(&self) -> MatchingSet {
        let mut matching_set = MatchingSet::default();
        for (i, bigs) in self.little_preferences.iter().enumerate() {
            self.update_matching(&mut matching_set, Index::from(i), bigs.iter());
        }
        matching_set
    }

    ///
    #[inline]
    pub fn find_primitive_matching(&self) -> MatchingSet {
        let mut matching_set = self.primitive_matching();
        self.collect_unmatched_bigs(&mut matching_set);
        matching_set
    }

    ///
    #[inline]
    pub fn find_even_matching(&self) -> Option<MatchingSet> {
        let mut matching_set = self.primitive_matching();
        let mut preference_index = IndexMap::<LittleIndex, usize>::new();
        while let Some(matching) = matching_set.largest_match(self.big_preferences.len()) {
            if let Some(little) = matching.littles.pop() {
                let preference_index = match preference_index.entry(little) {
                    Entry::Occupied(mut entry) => {
                        let index = *entry.get();
                        entry.insert(index + 1);
                        index
                    }
                    Entry::Vacant(entry) => {
                        let index = 1;
                        entry.insert(index);
                        index
                    }
                };
                self.update_matching(
                    &mut matching_set,
                    little,
                    self.little_preferences[little.index as usize]
                        .iter()
                        .skip(preference_index),
                );
            } else {
                return None;
            }
        }
        self.collect_unmatched_bigs(&mut matching_set);
        Some(matching_set)
    }
}

/// Matching
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Matching {
    /// Big Index
    pub big: BigIndex,

    /// Little Indices
    pub littles: IndexSet<LittleIndex>,
}

impl Matching {
    ///
    #[inline]
    pub fn from_pair(big: BigIndex, little: LittleIndex) -> Self {
        let mut littles = IndexSet::with_capacity(1);
        littles.insert(little);
        Self { big, littles }
    }
}

/// Matching Set
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MatchingSet {
    /// Matches
    matches: Vec<Matching>,

    /// Unmatched Bigs
    unmatched_bigs: IndexSet<BigIndex>,

    /// Unmatched Littles
    unmatched_littles: IndexSet<LittleIndex>,
}

impl MatchingSet {
    ///
    #[inline]
    fn insert_match(&mut self, table: &PreferenceTable, big: BigIndex, little: LittleIndex) {
        match self.matches.binary_search_by_key(&big, |m| m.big) {
            Ok(index) => {
                let littles = &mut self.matches[index].littles;
                littles.insert(little);
                littles.sort_by(|lhs, rhs| {
                    match (big.preference(*lhs, table), big.preference(*rhs, table)) {
                        (Some(lhs_preference), Some(rhs_preference)) => {
                            lhs_preference.cmp(&rhs_preference)
                        }
                        (None, Some(_)) => Ordering::Greater,
                        (Some(_), None) => Ordering::Less,
                        _ => Ordering::Equal,
                    }
                });
            }
            Err(index) => {
                self.matches.insert(index, Matching::from_pair(big, little));
            }
        }
    }

    ///
    #[inline]
    fn largest_match(&mut self, big_count: usize) -> Option<&mut Matching> {
        if self.matches.len() == big_count {
            return None;
        }
        if self.matches.len() == 1 {
            return self.matches.get_mut(0);
        }
        let first = self.matches.first()?.littles.len();
        if self
            .matches
            .iter()
            .skip(1)
            .all(|m| first == m.littles.len())
        {
            return None;
        }
        self.matches.iter_mut().reduce(|lhs, rhs| {
            if lhs.littles.len() < rhs.littles.len() {
                rhs
            } else {
                lhs
            }
        })
    }

    ///
    #[inline]
    pub fn display<'s>(&'s self, names: &'s Names) -> MatchingSetDisplay<'s> {
        MatchingSetDisplay {
            matching_set: self,
            names,
        }
    }
}

///
pub struct MatchingSetDisplay<'s> {
    ///
    matching_set: &'s MatchingSet,

    ///
    names: &'s Names,
}

impl<'s> fmt::Display for MatchingSetDisplay<'s> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MatchingSet {{\n    matches: [")?;
        for matching in &self.matching_set.matches {
            write!(
                f,
                "\n        {{ big: {:?}, littles: [",
                self.names.get(matching.big).unwrap(),
            )?;
            for little in &matching.littles {
                write!(f, "{:?}, ", self.names.get(*little).unwrap())?;
            }
            write!(f, "] }}")?;
        }
        writeln!(f, "\n    ],")?;
        write!(f, "    unmatched_bigs: [")?;
        for big in &self.matching_set.unmatched_bigs {
            write!(f, "{:?}, ", self.names.get(*big).unwrap())?;
        }
        writeln!(f, "]")?;
        write!(f, "    unmatched_littles: [")?;
        for little in &self.matching_set.unmatched_littles {
            write!(f, "{:?}, ", self.names.get(*little).unwrap())?;
        }
        writeln!(f, "],")?;
        write!(f, "}}")
    }
}
