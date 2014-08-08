#![crate_name = "scenegraph"]
#![comment = "An experimental scenegraph on top of gfx-rs"]
#![license = "ASL2"]
#![crate_type = "lib"]

#![feature(macro_rules)]

use std::slice;

pub mod macros;

type IdType = u32;

#[deriving(Clone, PartialEq, Show)]
pub struct Id<S>(IdType);

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

	pub fn change(&mut self, Id(i): Id<T>) -> &mut T {
		let Array(ref mut a) = *self;
		a.get_mut(i as uint)
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
