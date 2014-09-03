//! GLTF package loader for gfx-rs

#![crate_name = "gltf"]

extern crate serialize;
extern crate cgmath;
extern crate gfx;

use serialize::json;
use std::from_str::FromStr;

pub enum LoadError {
    ErrorJson,
}

pub struct Package {
    pub context: gfx::batch::Context,
}

impl Package {
    fn load(input: &str) -> Result<Package, LoadError> {
        let json: json::Json = match FromStr::from_str(input) {
            Some(j) => j,
            None => return Err(ErrorJson)
        };
        Ok(Package {
            context: gfx::batch::Context::new()
        })
    }
}
