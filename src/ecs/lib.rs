//! Entity-Component System

#![crate_name = "ecs"]
#![feature(macro_rules)]

use std::{fmt, slice};

pub mod macros;
#[cfg(test)]
pub mod test;

type IdType = u32;

// Deriving forces T to have the same properties, we can't afford that.
//#[deriving(Clone, Eq, Ord, PartialEq, PartialOrd, Show)]
pub struct Id<S>(IdType);

impl<S> Id<S> {
    fn unwrap(&self) -> IdType {
        let Id(i) = *self;
        i
    }
}

impl<S> Clone for Id<S> {
    fn clone(&self) -> Id<S> {
        Id(self.unwrap())
    }
}

impl<S> Eq for Id<S> {}

impl<S> Ord for Id<S> {
    fn cmp(&self, other: &Id<S>) -> Ordering {
        self.unwrap().cmp(&other.unwrap())
    }
}

impl<S> PartialEq for Id<S> {
    fn eq(&self, other: &Id<S>) -> bool {
        self.unwrap() == other.unwrap()
    }
}

impl<S> PartialOrd for Id<S> {
    fn partial_cmp(&self, other: &Id<S>) -> Option<Ordering> {
        self.unwrap().partial_cmp(&other.unwrap())
    }
}

impl<S> fmt::Show for Id<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Id({})", self.unwrap())
    }
}


#[deriving(Clone, Show)]
pub struct Array<T>(Vec<T>);

impl<T> Array<T> {
    pub fn new() -> Array<T> {
        Array(Vec::new())
    }

    pub fn add(&mut self, value: T) -> Id<T> {
        let Array(ref mut a) = *self;
        a.push(value);
        Id(a.len() as IdType - 1)
    }

    pub fn get(&self, Id(i): Id<T>) -> &T {
        let Array(ref a) = *self;
        &a[i as uint]
    }

    pub fn get_mut(&mut self, Id(i): Id<T>) -> &mut T {
        let Array(ref mut a) = *self;
        a.get_mut(i as uint)
    }

    pub fn find_id(&self, fun: |&T| -> bool) -> Option<Id<T>> {
        self.iter().position(fun).map(|i| Id(i as IdType))
    }

    pub fn iter<'a>(&'a self) -> slice::Items<'a, T> {
        let Array(ref a) = *self;
        a.iter()
    }

    pub fn mut_iter<'a>(&'a mut self) -> slice::MutItems<'a, T> {
        let Array(ref mut a) = *self;
        a.mut_iter()
    }
}
