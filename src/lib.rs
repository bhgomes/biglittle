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

/// Select Base
pub trait SelectBase<K>
where
    K: Kind,
{
    /// Select Type
    type Type;
}

/// Select Type Alias
pub type SelectType<K, S> = <S as SelectBase<K>>::Type;

/// Select
pub trait Select<K>: SelectBase<K> + SelectBase<Big> + SelectBase<Little>
where
    K: Kind,
{
    /// Returns a shared reference to the big variant of the [`SelectBase`] data structure.
    fn big(&self) -> &SelectType<Big, Self>;

    /// Returns a shared reference to the little variant of the [`SelectBase`] data structure.
    fn little(&self) -> &SelectType<Little, Self>;
}

/// Select Mutable
pub trait SelectMut<K>: Select<K>
where
    K: Kind,
{
    /// Returns a mutable reference to the big variant of the [`SelectBase`] data structure.
    fn big(&mut self) -> &mut SelectType<Big, Self>;

    /// Returns a mutable reference to the little variant of the [`SelectBase`] data structure.
    fn little(&mut self) -> &mut SelectType<Little, Self>;
}

/// Selection Kind
pub trait Kind: sealed::Sealed + Sized {
    /// Opposite Kind
    ///
    /// For bigs the opposite kind are littles, and for littles the opposite kind are bigs.
    type Opposite: Kind;

    /// Returns the [`DynamicKind`] that matches `Self`.
    fn dynamic() -> DynamicKind;

    /// Returns a shared reference to the `Self`-variant of `select`.
    fn select<S>(select: &S) -> &SelectType<Self, S>
    where
        S: Select<Self>;

    /// Returns a mutable reference to the `Self`-variant of `select`.
    fn select_mut<S>(select: &mut S) -> &mut SelectType<Self, S>
    where
        S: SelectMut<Self>;
}

/// Builds the implementation of a [`Kind`] marker.
macro_rules! impl_kind {
    ($doc:expr, $type:ident, $opposite:ident, $method:ident, $index:ident, $preference:ident) => {
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
            fn select<S>(select: &S) -> &SelectType<Self, S>
            where
                S: Select<Self>,
            {
                select.$method()
            }

            #[inline]
            fn select_mut<S>(select: &mut S) -> &mut SelectType<Self, S>
            where
                S: SelectMut<Self>,
            {
                select.$method()
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

impl_kind!("Big", Big, Little, big, BigIndex, BigPreference);
impl_kind!("Little", Little, Big, little, LittleIndex, LittlePreference);

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
        K::select(table)[self.index as usize]
            .iter()
            .position(|i| *i == other)
            .and_then(|i| NonZeroU32::new((i + 1) as u32).map(Preference::new))
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

/// Names
#[derive(Debug, Default)]
pub struct Names<T = String> {
    /// Big Names
    bigs: IndexSet<T>,

    /// Little Names
    littles: IndexSet<T>,
}

impl Names {
    /// Insert a new `name` with the given kind `K`. This method returns `None` if `name` is
    /// contained in the opposite variant.
    #[inline]
    pub fn insert<K>(&mut self, name: String) -> Option<Index<K>>
    where
        K: Kind,
    {
        if K::Opposite::select(self).contains(&name) {
            return None;
        }
        Some(K::select_mut(self).insert_full(name).0.into())
    }

    /// Returns the name associated to `index` if it is contained in the set.
    #[inline]
    pub fn get<K>(&self, index: Index<K>) -> Option<&String>
    where
        K: Kind,
    {
        K::select(self).get_index(index.index as usize)
    }

    /// Returns the index of the `name` in `self` if it has an entry of the kind `K`.
    #[inline]
    pub fn index<K>(&self, name: &str) -> Option<Index<K>>
    where
        K: Kind,
    {
        K::select(self).get_index_of(name).map(Into::into)
    }

    /// Finds the length of the longest name of kind `K`.
    #[inline]
    fn longest_name_length<K>(&self) -> usize
    where
        K: Kind,
    {
        K::select(self)
            .iter()
            .map(String::len)
            .max()
            .unwrap_or_default()
    }
}

impl<K> SelectBase<K> for Names
where
    K: Kind,
{
    type Type = IndexSet<String>;
}

impl<K> Select<K> for Names
where
    K: Kind,
{
    #[inline]
    fn big(&self) -> &SelectType<Big, Self> {
        &self.bigs
    }

    #[inline]
    fn little(&self) -> &SelectType<Little, Self> {
        &self.littles
    }
}

impl<K> SelectMut<K> for Names
where
    K: Kind,
{
    #[inline]
    fn big(&mut self) -> &mut SelectType<Big, Self> {
        &mut self.bigs
    }

    #[inline]
    fn little(&mut self) -> &mut SelectType<Little, Self> {
        &mut self.littles
    }
}

/// Matching Preference Table
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PreferenceTable {
    /// Big Preferences
    big_preferences: SelectType<Big, Self>,

    /// Little Preferences
    little_preferences: SelectType<Little, Self>,
}

impl PreferenceTable {
    /// Inserts the `preferences` as the next row in the preference table.
    #[inline]
    pub fn insert<K, I>(&mut self, preferences: I)
    where
        K: Kind,
        I: IntoIterator<Item = Index<K::Opposite>>,
    {
        K::select_mut(self).insert(Vec::from_iter(preferences));
    }

    /// Updates the `matching_set` by choosing from the preferences of `little` and seeing if any of
    /// the bigs in that ordering also prefer `little`. If not, the `little` is unmatched.
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

    /// Collects all the unmatched bigs relative to `matching_set` and declares them as unmatched.
    #[inline]
    fn collect_unmatched_bigs(&self, matching_set: &mut MatchingSet) {
        for big in 0..self.big_preferences.len() {
            let big = Index::from(big);
            if !matching_set.matches.iter().any(|m| m.big == big) {
                matching_set.unmatched_bigs.insert(big);
            }
        }
    }

    /// Finds the maximal matching. See [`find_maximal_matching`](Self::find_maximal_matching)
    /// for more.
    #[inline]
    fn maximal_matching(&self) -> MatchingSet {
        let mut matching_set = MatchingSet::default();
        for (i, bigs) in self.little_preferences.iter().enumerate() {
            self.update_matching(&mut matching_set, Index::from(i), bigs.iter());
        }
        matching_set
    }

    /// Finds the maximal matching where littles select according to their preferences and all bigs
    /// are assumed to have as much capacity as needed to accomodate all the littles they rank in
    /// their preferences.
    #[inline]
    pub fn find_maximal_matching(&self) -> MatchingSet {
        let mut matching_set = self.maximal_matching();
        self.collect_unmatched_bigs(&mut matching_set);
        matching_set
    }

    /// Finds the evenly-distributed matching.
    ///
    /// # Algorithm
    ///
    /// First a maximal matching is computed which fills all bigs to the capacity equal to the
    /// number of preferences they allocated. Then, if there are any bigs without matches, the
    /// fullest big sends their least prefered little to look for another match, and the little
    /// proceeds down their ranking list to find the next big to match with. This continues until
    /// all bigs have at least one match or all the matches have an equal number of littles,
    /// whichever comes first.
    #[inline]
    pub fn find_even_matching(&self) -> MatchingSet {
        let mut matching_set = self.maximal_matching();
        let mut preference_index = IndexMap::<LittleIndex, usize>::new();
        while let Some(matching) = matching_set.next_largest_match(self.big_preferences.len()) {
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
                break;
            }
        }
        self.collect_unmatched_bigs(&mut matching_set);
        matching_set
    }

    /// Returns a [`Display`](fmt::Display) implementation for `self` which substitutes `names` for
    /// indices in the preference table.
    #[inline]
    pub fn display<'s>(&'s self, names: &'s Names) -> PreferenceTableDisplay<'s> {
        PreferenceTableDisplay { table: self, names }
    }
}

impl<K> SelectBase<K> for PreferenceTable
where
    K: Kind,
{
    type Type = IndexSet<Vec<Index<<K as Kind>::Opposite>>>;
}

impl<K> Select<K> for PreferenceTable
where
    K: Kind,
{
    #[inline]
    fn big(&self) -> &SelectType<Big, Self> {
        &self.big_preferences
    }

    #[inline]
    fn little(&self) -> &SelectType<Little, Self> {
        &self.little_preferences
    }
}

impl<K> SelectMut<K> for PreferenceTable
where
    K: Kind,
{
    #[inline]
    fn big(&mut self) -> &mut SelectType<Big, Self> {
        &mut self.big_preferences
    }

    #[inline]
    fn little(&mut self) -> &mut SelectType<Little, Self> {
        &mut self.little_preferences
    }
}

/// Preference Table Display
#[derive(Clone, Copy, Debug)]
pub struct PreferenceTableDisplay<'s> {
    /// Preference Table
    table: &'s PreferenceTable,

    /// Names
    names: &'s Names,
}

impl<'s> fmt::Display for PreferenceTableDisplay<'s> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let longest_big_name_length = self.names.longest_name_length::<Big>();
        let longest_little_name_length = self.names.longest_name_length::<Little>();
        write!(f, "PreferenceTable {{\n    bigs: {{")?;
        for (big, littles) in self.table.big_preferences.iter().enumerate() {
            let big_name = self.names.get(BigIndex::from(big)).unwrap();
            write!(
                f,
                "\n        {}: {}[",
                big_name,
                " ".repeat(longest_big_name_length - big_name.len())
            )?;
            display_iter(f, littles.iter().map(|i| self.names.get(*i).unwrap()))?;
            write!(f, "],")?;
        }
        write!(f, "\n    }},\n    littles: {{")?;
        for (little, bigs) in self.table.little_preferences.iter().enumerate() {
            let little_name = self.names.get(LittleIndex::from(little)).unwrap();
            write!(
                f,
                "\n        {}: {}[",
                little_name,
                " ".repeat(longest_little_name_length - little_name.len())
            )?;
            display_iter(f, bigs.iter().map(|i| self.names.get(*i).unwrap()))?;
            write!(f, "],")?;
        }
        write!(f, "\n    }},\n}}")?;
        Ok(())
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
    /// Starts a new [`Matching`] from `big` and the first matching `little`.
    #[inline]
    pub fn from_pair(big: BigIndex, little: LittleIndex) -> Self {
        let mut littles = IndexSet::with_capacity(1);
        littles.insert(little);
        Self { big, littles }
    }

    /// Inserts `little` into the matching and sorts the matching according to the big's preferences
    /// from `table`.
    #[inline]
    fn insert(&mut self, table: &PreferenceTable, little: LittleIndex) {
        self.littles.insert(little);
        self.sort(table);
    }

    /// Sorts the littles in `self` according to the big's preferences from `table`.
    #[inline]
    fn sort(&mut self, table: &PreferenceTable) {
        self.littles.sort_by(|lhs, rhs| {
            match (
                self.big.preference(*lhs, table),
                self.big.preference(*rhs, table),
            ) {
                (Some(lhs_preference), Some(rhs_preference)) => lhs_preference.cmp(&rhs_preference),
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                _ => Ordering::Equal,
            }
        });
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
    /// Inserts the `big`-`little` match into `self`, sorting the existing match by the preference
    /// `table` according to the `big`.
    #[inline]
    fn insert_match(&mut self, table: &PreferenceTable, big: BigIndex, little: LittleIndex) {
        match self.matches.binary_search_by_key(&big, |m| m.big) {
            Ok(index) => self.matches[index].insert(table, little),
            Err(index) => self.matches.insert(index, Matching::from_pair(big, little)),
        }
    }

    /// Finds the next largest matching in `self` which should remove its lowest ranking little.
    /// This method is the exiting condition for the [`PreferenceTable::find_even_matching`] method.
    ///
    /// # Exit Conditions
    ///
    /// This method returns `None` on the following conditions (in this order):
    ///
    /// 1. The number of matches is equal to the number of bigs.
    /// 2. There are no matches whatsoever.
    /// 3. All the matches have the same number of littles.
    #[inline]
    fn next_largest_match(&mut self, big_count: usize) -> Option<&mut Matching> {
        if self.matches.len() == big_count {
            return None;
        }
        if self.matches.len() == 1 {
            return self.matches.get_mut(0);
        }
        let first_len = self.matches.first()?.littles.len();
        if self
            .matches
            .iter()
            .skip(1)
            .all(|m| first_len == m.littles.len())
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

    /// Returns a [`Display`](fmt::Display) implementation for `self` which substitutes `names` for
    /// indices in the matching set.
    #[inline]
    pub fn display<'s>(&'s self, names: &'s Names) -> MatchingSetDisplay<'s> {
        MatchingSetDisplay {
            matching_set: self,
            names,
        }
    }
}

/// Matching Set Display
#[derive(Clone, Copy, Debug)]
pub struct MatchingSetDisplay<'s> {
    /// Matching Set
    matching_set: &'s MatchingSet,

    /// Names
    names: &'s Names,
}

impl<'s> fmt::Display for MatchingSetDisplay<'s> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let longest_big_name_length = self.names.longest_name_length::<Big>();
        write!(f, "MatchingSet {{\n    matches: {{")?;
        for matching in &self.matching_set.matches {
            let big_name = self.names.get(matching.big).unwrap();
            write!(
                f,
                "\n        {}: {}[",
                big_name,
                " ".repeat(longest_big_name_length - big_name.len())
            )?;
            display_iter(
                f,
                matching.littles.iter().map(|i| self.names.get(*i).unwrap()),
            )?;
            write!(f, "],")?;
        }
        writeln!(f, "\n    }},")?;
        write!(f, "    unmatched_bigs:    [")?;
        display_iter(
            f,
            self.matching_set
                .unmatched_bigs
                .iter()
                .map(|i| self.names.get(*i).unwrap()),
        )?;
        writeln!(f, "]")?;
        write!(f, "    unmatched_littles: [")?;
        display_iter(
            f,
            self.matching_set
                .unmatched_littles
                .iter()
                .map(|i| self.names.get(*i).unwrap()),
        )?;
        writeln!(f, "],")?;
        write!(f, "}}")
    }
}

/// Displays an iterator by adding commas between each element.
#[inline]
fn display_iter<'t, T, I>(f: &mut fmt::Formatter, iter: I) -> fmt::Result
where
    T: 't + fmt::Display,
    I: IntoIterator<Item = &'t T>,
{
    let mut iter = iter.into_iter().peekable();
    while let Some(next) = iter.next() {
        if iter.peek().is_some() {
            write!(f, "{}, ", next)?;
        } else {
            write!(f, "{}", next)?;
        }
    }
    Ok(())
}
