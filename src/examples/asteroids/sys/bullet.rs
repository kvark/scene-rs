use cgmath::angle::{Angle, Rad};
use cgmath::point::Point;
use cgmath::vector::Vector;
use es = scenegraph;
use w = world;

pub struct System {
	pub shoot: bool,
	ship_space_id: es::Id<w::Spatial>,
	ship_inertia_id: es::Id<w::Inertial>,
	draw: w::Drawable,
	cool_time: f32,
	pool: Vec<w::Entity>,
}

impl System {
	pub fn new(space_id: es::Id<w::Spatial>, inertia_id: es::Id<w::Inertial>, draw: w::Drawable) -> System {
		System {
			shoot: false,
			ship_space_id: space_id,
			ship_inertia_id: inertia_id,
			draw: draw,
			cool_time: 1.0,
			pool: Vec::new(),
		}
	}

	pub fn process(&mut self, delta: f32, hub: &mut w::DataHub, entities: &mut Vec<w::Entity>) {
		self.cool_time = if self.cool_time > delta {self.cool_time - delta} else {0.0};
		if self.shoot && self.cool_time <= 0.0 {
			self.cool_time = 0.2;
			let velocity = 5.0f32;
			let bullet = w::Bullet {
				life_time: Some(1.0f32),
			};
			let (space, inertia) = {
				let e_space = hub.space.get(self.ship_space_id);
				let e_inertia = hub.inertia.get(self.ship_inertia_id);
				(w::Spatial {
					pos: e_space.pos,
					orient: Rad{ s: 0.0 },
					scale: 0.1,
				}, w::Inertial {
					velocity: e_inertia.velocity + e_space.get_direction().mul_s(velocity),
					angular_velocity: Rad{ s: 0.0 },
				})
			};
			let ent = match self.pool.pop() {
				Some(ent) => {
					*hub.bullet.get_mut(ent.bullet.unwrap()) = bullet;
					*hub.space.get_mut(ent.space.unwrap()) = space;
					*hub.inertia.get_mut(ent.inertia.unwrap()) = inertia;
					ent
				},
				None => {
					hub.add()
						.space(space)
						.inertia(inertia)
						.draw(self.draw)
						.bullet(bullet)
						.entity
				},
			};
			entities.push(ent);
		}
		let (new_entities, reserve) = entities.partitioned(|ent| {
			match ent.bullet {
				Some(b_id) => {
					let bullet = hub.bullet.get_mut(b_id);
					match bullet.life_time {
						Some(ref mut t) if *t>delta => {
							*t -= delta;
							true
						},
						Some(_) => {
							bullet.life_time = None;
							false
						},
						None => true,
					}
				},
				None => true,
			}
		});
		*entities = new_entities;
		self.pool.push_all_move(reserve);
	}
}
