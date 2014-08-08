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
	let mut ent = Entity::new();
	hub.add(&mut ent).simple(4);
	println!("{}", hub.get(&ent).simple());
}
