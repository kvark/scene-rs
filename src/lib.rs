#![crate_name = "scenegraph"]
#![comment = "An experimental scenegraph on top of gfx-rs"]
#![license = "ASL2"]
#![crate_type = "lib"]

#![feature(macro_rules)]

pub mod ces;
pub mod macros;
#[cfg(test)]
pub mod test;
