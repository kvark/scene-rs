use cgmath::angle::{Angle, Rad};
use cgmath::point::Point;
use cgmath::vector::Vector;
use w = world;

pub struct System {
	pub thrust: f32,
	pub rotate: f32,
}

impl System {
	pub fn new() -> System {
		System {
			thrust: 0.0,
			rotate: 0.0,
		}
	}
	pub fn process(&mut self, delta: f32, hub: &mut w::DataHub, entities: &[w::Entity]) {
		for ent in entities.iter() {
			match (ent.control, ent.inertia) {
				(Some(c_id), Some(i_id)) => {
					let c = hub.control.get(c_id);
					let i = hub.inertia.get_mut(i_id);
					let rotate = delta * c.rotate_scale * self.rotate;
					i.angular_velocity = Rad{ s: rotate };
					match ent.space {
						Some(s_id) => {
							let s = hub.space.get_mut(s_id);
							let dir = s.get_direction();
							let thrust = delta * c.thrust_scale * self.thrust;
							i.velocity.add_self_v(&dir.mul_s(thrust));
						},
						None => (),
					}
				},
				(_, _) => (),
			}
		}
	}
}
