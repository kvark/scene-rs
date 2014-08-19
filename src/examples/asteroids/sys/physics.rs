use w = world;

pub struct System;

impl System {
	pub fn new() -> System {
		System
	}
}

impl w::System for System {
	fn process(&mut self, &(time, _): w::Params, data: &mut w::Components, entities: &mut Vec<w::Entity>) {
		for i in range(0, entities.len()) {
			for j in range(i+1, entities.len()) {
				
			}
		}
	}
}
