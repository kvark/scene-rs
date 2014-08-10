#![feature(phase)]
#![crate_name = "simple"]

#[phase(plugin, link)]
extern crate scenegraph;

pub type SimpleComponent = int;
entity! { scenegraph
	simple : SimpleComponent,
}

fn main() {
	let mut hub = DataHub::new();
	let ent = hub.add().simple(4).entity;
	println!("{}", hub.simple.get(ent.simple.unwrap()));
}
