//! Big-Little Matching

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![forbid(rustdoc::broken_intra_doc_links)]
#![forbid(missing_docs)]

extern crate alloc;

use alloc::vec::Vec;
use core::{marker::PhantomData, num::NonZeroU32};
use indexmap::IndexSet;

/// Sealed Module
mod sealed {
    /// Sealed Trait
    pub trait Sealed {}
}

/// Matching Kind
pub trait Kind: sealed::Sealed + Sized {
    /// Opposite Kind
    type Opposite: Kind;

    /// Selects the subset of the preference `table` which corresponds to `Self` preferences.
    fn preferences(table: &PreferenceTable) -> PreferenceSubset<Self>;
}

/// Matching Index
#[derive(derivative::Derivative)]
#[derivative(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
    pub fn max<I>(
        self,
        others: I,
        table: &PreferenceTable,
    ) -> Option<(Index<K::Opposite>, Preference<K>)>
    where
        I: IntoIterator<Item = Index<K::Opposite>>,
    {
        /* TODO:
        others
            .into_iter()
            .map(|i| self.preference(i, table))
            .max()
            .flatten()
        */
        todo!()
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
    pub fn find_matching(&self) -> Option<MatchingSet> {
        todo!()
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
    pub fn is_stable(&self, preferences: &PreferenceTable) -> bool {
        self.deviation_from_stability(preferences) == 0
    }

    ///
    #[inline]
    pub fn deviation_from_stability(&self, preferences: &PreferenceTable) -> u32 {
        todo!()
    }
}
