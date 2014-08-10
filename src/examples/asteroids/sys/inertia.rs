use cgmath::angle::Angle;
use cgmath::point::Point;
use cgmath::vector::Vector;
use w = world;

pub struct System;

impl System {
	pub fn process(&mut self, delta: f32, hub: &mut w::DataHub, entities: &[w::Entity]) {
		for ent in entities.iter() {
			ent.space.map(|s_id| {
				let s = hub.space.get_mut(s_id);
				ent.inertia.map(|i_id| {
					let i = hub.inertia.get(i_id);
					let move = i.velocity.mul_s(delta);
					s.pos.add_self_v(&move);
					s.orient.add_self_a(i.angular_velocity.mul_s(delta));
				});
			});
		}
	}
}
