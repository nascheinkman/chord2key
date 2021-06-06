use bitvec::prelude::*;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::iter::Iterator;
use std::ptr;
use std::rc::Rc;

/// An immutable set of tracked items
///
/// This is analogous to a constant, heap-allocated [std::collections::HashSet]. As such, it
/// requires its items to implement the [Eq] and [Hash] traits.
///
/// It allows for efficient, hashable subsets ([AttributeSubset]) of this set to be created and used
/// for as long as this set is still around.
///
/// # Examples:
/// ```
/// use chord2key::attribute_set::*;
/// use std::collections::HashMap;
/// use std::rc::Rc;
///
/// #[derive(PartialEq, Eq, Hash)]
/// enum Letters {
///     A, B, C, D
/// }
///
/// struct Combos {
///     pub letters: Rc<AttributeSet<Letters>>,
///     pub combo_map: HashMap<AttributeSubset<Letters>, i32>,
/// }
///
/// let mut combos = Combos {
///     letters: AttributeSet::<Letters>::from(vec![Letters::A, Letters::B, Letters::C]),
///     combo_map: HashMap::<AttributeSubset<Letters>, i32>::new(),
/// };
///
/// let combo_set1 = combos.letters.subset_with(vec![Letters::A, Letters::B]);
/// let combo_set2 = combos.letters.subset_with(vec![Letters::B, Letters::C]);
/// combos.combo_map.insert(combo_set1, 1);
/// combos.combo_map.insert(combo_set2, 2);
///
/// let combo_num = combos.combo_map.get(&combos.letters.subset_with(vec![Letters::B,Letters::C]));
/// assert_eq!(*combo_num.unwrap(), 2);
/// ```
///
/// It is a logic error for the AttributeSet or any of its items to be modified.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct AttributeSet<T>
where
    T: Hash + Eq,
{
    indexes: HashMap<T, usize>,
}

impl<T: Hash + Eq> AttributeSet<T> {
    /// Creates a new AttributeSet with a specified capacity for memory allocation
    ///
    ///# Example:
    /// ```
    /// use chord2key::attribute_set::*;
    ///
    /// let vec_set = vec![-9, -9, 7, 2, 5];
    /// let size = vec_set.len();
    /// let attr_set = AttributeSet::<i32>::from_capacity(vec_set, size);
    /// ```
    pub fn from_capacity<U>(set: U, capacity: usize) -> Rc<Self>
    where
        U: IntoIterator<Item = T>,
    {
        // First place everything in an intermediate hashset to remove duplicates
        let mut hashset = HashSet::<T>::with_capacity(capacity);

        set.into_iter().for_each(|item| {
            hashset.insert(item);
        });

        // Then place everything into the data structure's HashMap
        let mut indexes = HashMap::<T, usize>::with_capacity(hashset.len());

        hashset.into_iter().enumerate().for_each(|(i, val)| {
            indexes.insert(val, i);
        });
        Rc::new(Self { indexes: indexes })
    }

    /// Creates a new AttributeSet from a set U that contains items T for the AttributeSet.
    ///
    ///# Example:
    /// ```
    /// use chord2key::attribute_set::*;
    ///
    /// let vec_set = vec![5, 2, 4];
    /// let attr_set = AttributeSet::<i32>::from(vec_set);
    /// ```
    pub fn from<U>(set: U) -> Rc<Self>
    where
        U: IntoIterator<Item = T>,
    {
        Self::from_capacity(set, 0)
    }

    /// Returns the number of items that are contained in the AttributeSet.
    ///
    ///# Example:
    /// ```
    /// use chord2key::attribute_set::*;
    ///
    /// let vec_set = vec![5, 2, 4];
    /// let attr_set = AttributeSet::<i32>::from(vec_set);
    /// assert_eq!(attr_set.len(), 3);
    /// ```
    pub fn len(&self) -> usize {
        self.indexes.len()
    }

    /// Returns true if the set contains an item
    ///
    /// The item may be any borrowed form of the set's item type, but [Hash] and [Eq] on the
    /// borrowed form _must_ match those for the item type.
    ///
    ///# Example:
    /// ```
    /// use chord2key::attribute_set::*;
    ///
    /// let vec_set = vec![5, 2, 4];
    /// let attr_set = AttributeSet::<i32>::from(vec_set);
    /// assert_eq!(attr_set.contains(&4), true);
    /// assert_eq!(attr_set.contains(&3), false);
    /// ```
    pub fn contains(&self, value: &T) -> bool {
        self.indexes.contains_key(value)
    }

    /// Creates an empty [AttributeSubset] with this AttributeSet as its parent
    ///
    ///# Example:
    /// ```
    /// use chord2key::attribute_set::*;
    /// use std::rc::Rc;
    ///
    /// let attr_set = Rc::new(AttributeSet::<i32>::from(vec![5, 2, 4]));
    /// let subset = attr_set.empty_subset();
    /// ```
    pub fn empty_subset(self: &Rc<Self>) -> AttributeSubset<T> {
        AttributeSubset::<T>::from(self)
    }

    /// Creates a prefilled [AttributeSubset] with this AttributeSet as its parent. Consumes the
    /// subset source collection.
    ///
    ///# Example:
    /// ```
    /// use chord2key::attribute_set::*;
    /// use std::rc::Rc;
    ///
    /// let attr_set = Rc::new(AttributeSet::<i32>::from(vec![5, 2, 4]));
    /// let subset = attr_set.subset_with(vec![5, 2]);
    /// assert_eq!(subset.contains(&5), true);
    /// assert_eq!(subset.contains(&2), true);
    /// assert_eq!(subset.contains(&4), false);
    /// ```
    pub fn subset_with<U>(self: &Rc<Self>, items: U) -> AttributeSubset<T>
    where
        U: IntoIterator<Item = T>,
    {
        let mut subset = self.empty_subset();
        for item in items {
            subset.try_insert(&item).ok();
        }
        subset
    }

    /// Creates a prefilled [AttributeSubset] with this AttributeSet as its parent. Does not
    /// consume the subset source collection.
    ///
    ///# Example:
    /// ```
    /// use chord2key::attribute_set::*;
    /// use std::collections::HashSet;
    /// use std::rc::Rc;
    ///
    /// let attr_set = Rc::new(AttributeSet::<i32>::from(vec![5, 2, 4]));
    ///
    /// let mut unhashable_subset = HashSet::<i32>::new();
    /// unhashable_subset.insert(5);
    /// unhashable_subset.insert(2);
    ///
    /// let hashable_subset = attr_set.subset_from(unhashable_subset.iter());
    ///
    /// assert_eq!(hashable_subset.contains(&5), true);
    /// assert_eq!(hashable_subset.contains(&2), true);
    /// assert_eq!(hashable_subset.contains(&4), false);
    /// ```
    pub fn subset_from<'a, U>(self: &Rc<Self>, items: U) -> AttributeSubset<T>
    where
        U: Iterator<Item = &'a T>,
        T: 'a,
    {
        let mut subset = self.empty_subset();
        for item in items {
            subset.try_insert(&item).ok();
        }
        subset
    }

    /// An iterator visiting all the items in arbitrary order.
    ///
    ///# Example:
    /// ```
    /// use chord2key::attribute_set::*;
    /// use std::rc::Rc;
    ///
    /// let attr_set = Rc::new(AttributeSet::<i32>::from(vec![5, 2, 4]));
    /// let sum: i32 = attr_set.iter().sum();
    /// let max: i32 = *attr_set.iter().max().unwrap();
    /// assert_eq!(sum, 5 + 2 + 4);
    /// assert_eq!(max, 5);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.indexes.keys()
    }

    fn index_of(&self, value: &T) -> Option<&usize> {
        self.indexes.get(value)
    }

    fn map_iter(&self) -> impl Iterator<Item = (&T, &usize)> {
        self.indexes.iter()
    }
}

/// A compressed hashable, mutable subset of a specific [AttributeSet].
///
/// This is analogous to a mutable heap-allocated subset of a constant heap-allocated
/// [std::collections::HashSet]. As such, it requires its items to implement the [Eq] and [Hash]
/// traits.
///
///# Example
/// ```
/// use chord2key::attribute_set::*;
/// use std::collections::HashMap;
/// use std::rc::Rc;
///
/// #[derive(PartialEq, Eq, Hash)]
/// enum Letters {
///     A, B, C, D
/// }
///
/// struct Combos {
///     pub letters: Rc<AttributeSet<Letters>>,
///     pub combo_map: HashMap<AttributeSubset<Letters>, i32>,
/// }
///
/// let mut combos = Combos {
///     letters: AttributeSet::<Letters>::from(vec![Letters::A, Letters::B, Letters::C]),
///     combo_map: HashMap::<AttributeSubset<Letters>, i32>::new(),
/// };
///
/// let combo_set1 = combos.letters.subset_with(vec![Letters::A, Letters::B]);
/// let combo_set2 = combos.letters.subset_with(vec![Letters::B, Letters::C]);
/// combos.combo_map.insert(combo_set1, 1);
/// combos.combo_map.insert(combo_set2, 2);
///
/// let combo_num = combos.combo_map.get(&combos.letters.subset_with(vec![Letters::B,Letters::C]));
/// assert_eq!(*combo_num.unwrap(), 2);
/// ```
///
/// Note: the hashing and equality implementations depend on the subset being created from the same
/// parent in memory, not a clone.
///
///# Example:
/// ```
/// use chord2key::attribute_set::*;
/// use std::rc::Rc;
///
/// let set1 = AttributeSet::<i32>::from(vec![5, 9, 12]);
/// let set2 = AttributeSet::<i32>::from(vec![5, 9, 12]);
///
/// let subset1 = set1.subset_with(vec![5, 9]);
/// let subset2 = set2.subset_with(vec![5, 9]);
/// assert!(subset1 != subset2);
/// ```
pub struct AttributeSubset<T>
where
    T: Hash + Eq,
{
    parent: Rc<AttributeSet<T>>,
    items: BitVec,
}

impl<T> PartialEq for AttributeSubset<T>
where
    T: Hash + Eq,
{
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.parent, &other.parent) && self.items == other.items
    }
}

impl<T> Eq for AttributeSubset<T> where T: Hash + Eq {}

impl<T> Hash for AttributeSubset<T>
where
    T: Hash + Eq,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(Rc::as_ptr(&self.parent), state);
        self.items.hash(state);
    }
}

impl<T> AttributeSubset<T>
where
    T: Hash + Eq,
{
    pub(self) fn from(set: &Rc<AttributeSet<T>>) -> Self {
        let items = BitVec::repeat(false, set.len());
        Self {
            parent: set.clone(),
            items: items,
        }
    }

    /// Adds an item from the parent set to the subset
    ///
    /// If the item was successfully added (part of the parent set), returns Ok.
    ///
    /// If the item could not be added (not part of the parent set), returns an Error.
    ///
    ///# Example:
    /// ```
    /// use chord2key::attribute_set::*;
    /// use std::rc::Rc;
    ///
    /// let attr_set = Rc::new(AttributeSet::<i32>::from(vec![5, 2, 4]));
    /// let mut subset = attr_set.empty_subset();
    /// assert!(subset.try_insert(&5).is_ok());
    /// assert!(subset.try_insert(&3).is_err());
    /// ```
    pub fn try_insert(&mut self, item: &T) -> Result<(), &'static str> {
        let index = self.parent.index_of(item);
        match index {
            Some(index) => {
                *self.items.get_mut(*index).unwrap() = true;
                Ok(())
            }
            None => Err("Could not add the item -- it was not part of the parent set"),
        }
    }

    /// Removes an item from the subset
    ///
    ///# Example:
    /// ```
    /// use chord2key::attribute_set::*;
    /// use std::rc::Rc;
    ///
    /// let attr_set = Rc::new(AttributeSet::<i32>::from(vec![5, 2, 4]));
    /// let mut subset = attr_set.empty_subset();
    ///
    /// subset.try_insert(&5).ok();
    /// assert!(subset.contains(&5));
    ///
    /// subset.remove(&5);
    /// assert!(!subset.contains(&5));
    /// ```
    pub fn remove(&mut self, item: &T) {
        let index = self.parent.index_of(item);
        if let Some(index) = index {
            *self.items.get_mut(*index).unwrap() = false;
        }
    }

    /// Returns `true` if the set contains a item.
    ///
    /// The item may be any borrowed form of the set's item type, but [Hash] and [Eq] on the
    /// borrowed form must match those for the item type.
    ///
    ///# Example:
    /// ```
    /// use chord2key::attribute_set::*;
    ///
    /// let attr_set = AttributeSet::<i32>::from(vec![5, 2, 4]);
    /// let mut subset = attr_set.empty_subset();
    /// subset.try_insert(&5).ok();
    /// assert_eq!(subset.contains(&5), true);  // Contained by both parent set and subset
    /// assert_eq!(subset.contains(&4), false); // Contained by parent set but not subset
    /// assert_eq!(subset.contains(&3), false); // Contained by neither parent set nor subset
    /// ```
    pub fn contains(&self, item: &T) -> bool {
        let index = self.parent.index_of(item);
        match index {
            Some(index) => *self.items.get(*index).unwrap(),
            None => false,
        }
    }

    /// Clears the set, removing all items.
    ///
    ///# Example:
    /// ```
    /// use chord2key::attribute_set::*;
    ///
    /// let attr_set = AttributeSet::<i32>::from(vec![5, 2, 4]);
    /// let mut subset = attr_set.subset_with(vec![5, 2]);
    /// assert!(subset.contains(&5));
    /// assert!(subset.contains(&2));
    ///
    /// subset.clear();
    /// assert!(!subset.contains(&5));
    /// assert!(!subset.contains(&2));
    pub fn clear(&mut self) {
        self.items.set_all(false);
    }

    /// An iterator visiting all the items in the subset in arbitrary order.
    ///
    ///# Example:
    /// ```
    /// use chord2key::attribute_set::*;
    /// use std::rc::Rc;
    /// let attr_set = Rc::new(AttributeSet::<i32>::from(vec![5, 2, 4]));
    /// let subset = attr_set.subset_with(vec![5, 4]);
    ///
    /// let sum: i32 = subset.items().sum();
    /// assert_eq!(sum, 9);
    ///
    /// let min: i32 = *subset.items().min().unwrap();
    /// assert_eq!(min, 4);
    /// ```
    pub fn items<'b>(&self) -> impl Iterator<Item = &T> {
        let items = &self.items;
        self.parent
            .map_iter()
            .filter(move |(_item, i)| *items.get(**i).unwrap())
            .map(|(item, _i)| item)
    }

    /// Copies from a different AttributeSubset, returning a result if it was successful
    ///
    /// Returns an error if the AttributeSubset has a different parent
    ///
    /// # Example:
    /// ```
    /// use chord2key::attribute_set::*;
    /// use std::rc::Rc;
    ///
    /// let attr_set = Rc::new(AttributeSet::<i32>::from(vec![5, 2, 4]));
    ///
    /// let subset1 = attr_set.subset_with(vec![5, 4]);
    /// let mut subset2 = attr_set.empty_subset();
    ///
    /// assert!(!subset2.contains(&5));
    /// assert!(!subset2.contains(&4));
    ///
    /// subset2.copy_from(&subset1);
    ///
    /// assert!(subset2.contains(&5));
    /// assert!(subset2.contains(&4));
    /// assert!(!subset2.contains(&2));
    /// ```
    pub fn copy_from(&mut self, other: &AttributeSubset<T>) -> Result<(), &'static str> {
        if !Rc::ptr_eq(&self.parent, &other.parent) {
            return Err("The AttributeSubset does not spawn from the same parent set!");
        }
        self.items.copy_from_bitslice(&other.items);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicates() {
        let vec_set = vec![5, 5, 5, 5, 2, 4];
        let size = vec_set.len();
        let set = AttributeSet::<i32>::from_capacity(vec_set, size);
        assert_eq!(*set.map_iter().map(|(_num, index)| index).max().unwrap(), 2);
    }
}
