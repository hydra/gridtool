//! Identity for objects placed on a document (currently only measurements; grids and
//! area measurements will follow).

use std::fmt;
use std::ops::{Deref, DerefMut};

/// A stable identifier for a document object. Unlike a list position it does not change
/// when other objects are added or deleted, so selections can refer to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectId(u64);

impl ObjectId {
    pub(crate) fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub(crate) fn raw(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// An object stored in a document, paired with its [`ObjectId`]. Dereferences to the
/// object itself.
#[derive(Debug, Clone, PartialEq)]
pub struct Identified<T> {
    id: ObjectId,
    value: T,
}

impl<T> Identified<T> {
    pub(crate) fn new(id: ObjectId, value: T) -> Self {
        Self { id, value }
    }

    pub fn id(&self) -> ObjectId {
        self.id
    }
}

impl<T> Deref for Identified<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for Identified<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}
