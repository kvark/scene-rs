#![crate_name = "scenegraph"]
#![comment = "An experimental higher-level library on top of gfx-rs"]
#![license = "ASL2"]
#![crate_type = "lib"]

#![feature(macro_rules)]

pub mod ces;
#[cfg(test)]
pub mod tests {
    pub mod ces;
}
