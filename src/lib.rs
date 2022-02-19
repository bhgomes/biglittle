//! Big-Little Matching

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![forbid(rustdoc::broken_intra_doc_links)]
#![forbid(missing_docs)]

extern crate alloc;

use alloc::{string::String, vec::Vec};
use core::{marker::PhantomData, num::NonZeroU32};
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

/// Sealed Module
mod sealed {
    /// Sealed Trait
    pub trait Sealed {}
}

/// Dynamic Kind
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
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

    /// Returns a shared references to the subset of the preference `table` which corresponds to
    /// `Self` preferences.
    fn preferences(table: &PreferenceTable) -> PreferenceSubset<Self>;

    /// Returns a mutable references to the subset of the preference `table` which corresponds to
    /// `Self` preferences.
    fn preferences_mut(table: &mut PreferenceTable) -> PreferenceSubsetMut<Self>;
}

/// Matching Index
#[derive(derivative::Derivative)]
#[derivative(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
#[derivative(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
    pub fn find_matching(&self) -> Option<MatchingSet> {
        // TODO: This does not take into account spreading the distribution.

        let mut matching_set = MatchingSet::default();
        for (i, preferences) in self.little_preferences.iter().enumerate() {
            for big in preferences {
                matching_set.insert_match(*big, Index::new(i as u32));
            }
        }
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
    fn insert_match(&mut self, big: BigIndex, little: LittleIndex) {
        match self.matches.binary_search_by_key(&big, |m| m.big) {
            Ok(index) => {
                self.matches[index].littles.insert(little);
            }
            Err(index) => {
                self.matches.insert(index, Matching::from_pair(big, little));
            }
        }
    }

    ///
    #[inline]
    pub fn is_stable(&self, preferences: &PreferenceTable) -> bool {
        self.deviation_from_stability(preferences) == 0
    }

    ///
    #[inline]
    pub fn deviation_from_stability(&self, preferences: &PreferenceTable) -> u32 {
        todo!()
    }
}

///
#[derive(Debug, Default)]
pub struct Names {
    /// Name Set
    names: IndexSet<String>,
}

impl Names {
    ///
    #[inline]
    pub fn insert(&mut self, name: String) -> u32 {
        self.names.insert_full(name).0 as u32
    }

    ///
    #[inline]
    pub fn name(&self, index: u32) -> Option<&String> {
        self.names.get_index(index as usize)
    }

    ///
    #[inline]
    pub fn index(&self, name: &str) -> Option<u32> {
        self.names.get_index_of(name).map(|i| i as u32)
    }
}
